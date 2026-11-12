use crate::VideoFormat;
use crate::bits::BitReader;

const OBU_SEQUENCE_HEADER: u8 = 1;
const CONFIGURATION_MARKER_AND_VERSION: u8 = 0x81;

pub fn video_format(keyframe: &[u8]) -> Option<VideoFormat> {
    let mut rest = keyframe;
    while !rest.is_empty() {
        let obu_type = (rest[0] >> 3) & 0xF;
        let header_length = 1 + usize::from(rest[0] & 0x4 != 0);
        let (payload_length, size_length) = read_leb128(&rest[header_length..]);
        let payload_start = header_length + size_length;
        let obu_length = payload_start + payload_length;
        if obu_type == OBU_SEQUENCE_HEADER {
            return Some(parse_sequence_header(&rest[..obu_length], &rest[payload_start..obu_length]));
        }
        rest = &rest[obu_length..];
    }
    None
}

fn read_leb128(bytes: &[u8]) -> (usize, usize) {
    let mut value = 0;
    for (index, byte) in bytes.iter().enumerate() {
        value |= usize::from(byte & 0x7F) << (7 * index);
        if byte & 0x80 == 0 {
            return (value, index + 1);
        }
    }
    (value, bytes.len())
}

fn parse_sequence_header(obu: &[u8], payload: &[u8]) -> VideoFormat {
    let mut bits = BitReader::new(payload);
    let profile = bits.read(3);
    bits.skip(1);
    let is_reduced_still_picture_header = bits.read_flag();
    let (level, tier) = if is_reduced_still_picture_header {
        (bits.read(5), 0)
    } else {
        read_first_operating_point(&mut bits)
    };
    let width_bit_count = bits.read(4) + 1;
    let height_bit_count = bits.read(4) + 1;
    let width = bits.read(width_bit_count) + 1;
    let height = bits.read(height_bit_count) + 1;
    if !is_reduced_still_picture_header && bits.read_flag() {
        bits.skip(4 + 3);
    }
    bits.skip(3);
    if !is_reduced_still_picture_header {
        bits.skip(4);
        let has_order_hint = bits.read_flag();
        if has_order_hint {
            bits.skip(2);
        }
        let force_screen_content_tools = if bits.read_flag() { 2 } else { bits.read(1) };
        if force_screen_content_tools > 0 && !bits.read_flag() {
            bits.skip(1);
        }
        if has_order_hint {
            bits.skip(3);
        }
    }
    bits.skip(3);
    let color = read_color_config(&mut bits, profile);
    let flags = [
        tier,
        color.is_high_bitdepth.into(),
        color.is_twelve_bit.into(),
        color.is_monochrome.into(),
        color.subsampling_x,
        color.subsampling_y,
    ];
    let packed_flags = flags.iter().fold(0, |packed, flag| packed << 1 | flag) << 2 | color.chroma_sample_position;
    let mut codec_configuration = vec![CONFIGURATION_MARKER_AND_VERSION, (profile << 5 | level) as u8, packed_flags as u8, 0];
    codec_configuration.extend_from_slice(obu);
    VideoFormat::Av1 {
        width,
        height,
        codec_configuration,
    }
}

fn read_first_operating_point(bits: &mut BitReader) -> (u32, u32) {
    let has_timing_info = bits.read_flag();
    let mut has_decoder_model_info = false;
    let mut buffer_delay_bit_count = 0;
    if has_timing_info {
        bits.skip(64);
        if bits.read_flag() {
            bits.read_exp_golomb();
        }
        has_decoder_model_info = bits.read_flag();
        if has_decoder_model_info {
            buffer_delay_bit_count = bits.read(5) + 1;
            bits.skip(32 + 5 + 5);
        }
    }
    let has_initial_display_delay = bits.read_flag();
    let operating_point_count = bits.read(5) + 1;
    let mut first_operating_point = (0, 0);
    for index in 0..operating_point_count {
        bits.skip(12);
        let level = bits.read(5);
        let tier = if level > 7 { bits.read(1) } else { 0 };
        if has_decoder_model_info && bits.read_flag() {
            bits.skip(2 * buffer_delay_bit_count + 1);
        }
        if has_initial_display_delay && bits.read_flag() {
            bits.skip(4);
        }
        if index == 0 {
            first_operating_point = (level, tier);
        }
    }
    first_operating_point
}

struct ColorConfig {
    is_high_bitdepth: bool,
    is_twelve_bit: bool,
    is_monochrome: bool,
    subsampling_x: u32,
    subsampling_y: u32,
    chroma_sample_position: u32,
}

fn read_color_config(bits: &mut BitReader, profile: u32) -> ColorConfig {
    let is_high_bitdepth = bits.read_flag();
    let is_twelve_bit = profile == 2 && is_high_bitdepth && bits.read_flag();
    let is_monochrome = profile != 1 && bits.read_flag();
    let (primaries, transfer, matrix) = if bits.read_flag() {
        (bits.read(8), bits.read(8), bits.read(8))
    } else {
        (2, 2, 2)
    };
    let mut color = ColorConfig {
        is_high_bitdepth,
        is_twelve_bit,
        is_monochrome,
        subsampling_x: 1,
        subsampling_y: 1,
        chroma_sample_position: 0,
    };
    if is_monochrome {
        return color;
    }
    if primaries == 1 && transfer == 13 && matrix == 0 {
        color.subsampling_x = 0;
        color.subsampling_y = 0;
        return color;
    }
    bits.skip(1);
    match profile {
        0 => {}
        1 => {
            color.subsampling_x = 0;
            color.subsampling_y = 0;
        }
        _ if is_twelve_bit => {
            color.subsampling_x = bits.read(1);
            color.subsampling_y = if color.subsampling_x == 1 { bits.read(1) } else { 0 };
        }
        _ => color.subsampling_y = 0,
    }
    if color.subsampling_x == 1 && color.subsampling_y == 1 {
        color.chroma_sample_position = bits.read(2);
    }
    color
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_av1c_box_ffmpeg_writes_for_an_svt_av1_keyframe() {
        let keyframe = [
            0x12, 0x00, 0x0A, 0x0C, 0x02, 0x00, 0x00, 0x4A, 0x95, 0x5D, 0xFE, 0x1B, 0x8D, 0x5F, 0x30, 0x08, 0x32, 0xCC,
        ];
        let ffmpeg_av1c = [
            0x81, 0x09, 0x0C, 0x00, 0x0A, 0x0C, 0x02, 0x00, 0x00, 0x4A, 0x95, 0x5D, 0xFE, 0x1B, 0x8D, 0x5F, 0x30, 0x08,
        ];
        let Some(VideoFormat::Av1 { width, height, codec_configuration }) = video_format(&keyframe) else {
            panic!("no AV1 format");
        };
        assert_eq!((width, height), (1920, 1080));
        assert_eq!(codec_configuration, ffmpeg_av1c);
    }
}
