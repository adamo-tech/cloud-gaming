use crate::VideoFormat;
use crate::bits::BitReader;

const START_CODE: [u8; 3] = [0, 0, 1];
const NAL_SEQUENCE_PARAMETER_SET: u8 = 7;
const NAL_PICTURE_PARAMETER_SET: u8 = 8;
const PROFILES_WITH_CHROMA_FORMAT: [u32; 13] =
    [100, 110, 122, 244, 44, 83, 86, 118, 128, 138, 139, 134, 135];

pub fn is_annex_b(access_unit: &[u8]) -> bool {
    access_unit.starts_with(&START_CODE) || access_unit.starts_with(&[0, 0, 0, 1])
}

pub fn video_format(keyframe: &[u8]) -> Option<VideoFormat> {
    let nal_units = nal_units(keyframe);
    let find = |nal_type| nal_units.iter().find(|nal| nal[0] & 0x1F == nal_type);
    let sequence_parameter_set = find(NAL_SEQUENCE_PARAMETER_SET)?;
    let picture_parameter_set = find(NAL_PICTURE_PARAMETER_SET)?;
    let (width, height) = read_size(sequence_parameter_set);
    Some(VideoFormat::H264 {
        width,
        height,
        sequence_parameter_set: sequence_parameter_set.to_vec(),
        picture_parameter_set: picture_parameter_set.to_vec(),
    })
}

pub fn length_prefixed(access_unit: &[u8]) -> Vec<u8> {
    nal_units(access_unit)
        .into_iter()
        .flat_map(|nal| {
            (nal.len() as u32)
                .to_be_bytes()
                .into_iter()
                .chain(nal.iter().copied())
        })
        .collect()
}

fn nal_units(access_unit: &[u8]) -> Vec<&[u8]> {
    let starts: Vec<usize> = access_unit
        .windows(START_CODE.len())
        .enumerate()
        .filter(|(_, window)| *window == START_CODE)
        .map(|(index, _)| index + START_CODE.len())
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(position, &start)| {
            let end = starts
                .get(position + 1)
                .map_or(access_unit.len(), |next| next - START_CODE.len());
            let nal = &access_unit[start..end];
            let trailing_zero_count = nal.iter().rev().take_while(|byte| **byte == 0).count();
            &nal[..nal.len() - trailing_zero_count]
        })
        .collect()
}

fn read_size(sequence_parameter_set: &[u8]) -> (u32, u32) {
    let payload = remove_emulation_prevention(&sequence_parameter_set[1..]);
    let mut bits = BitReader::new(&payload);
    let profile = bits.read(8);
    bits.skip(16);
    bits.read_exp_golomb();
    let mut chroma_format = 1;
    if PROFILES_WITH_CHROMA_FORMAT.contains(&profile) {
        chroma_format = bits.read_exp_golomb();
        if chroma_format == 3 {
            bits.skip(1);
        }
        bits.read_exp_golomb();
        bits.read_exp_golomb();
        bits.skip(1);
        if bits.read_flag() {
            let list_count = if chroma_format == 3 { 12 } else { 8 };
            for index in 0..list_count {
                if bits.read_flag() {
                    skip_scaling_list(&mut bits, if index < 6 { 16 } else { 64 });
                }
            }
        }
    }
    bits.read_exp_golomb();
    match bits.read_exp_golomb() {
        0 => {
            bits.read_exp_golomb();
        }
        1 => {
            bits.skip(1);
            bits.read_signed_exp_golomb();
            bits.read_signed_exp_golomb();
            for _ in 0..bits.read_exp_golomb() {
                bits.read_signed_exp_golomb();
            }
        }
        _ => {}
    }
    bits.read_exp_golomb();
    bits.skip(1);
    let width_in_macroblocks = bits.read_exp_golomb() + 1;
    let height_in_map_units = bits.read_exp_golomb() + 1;
    let is_frame_only = bits.read_flag();
    if !is_frame_only {
        bits.skip(1);
    }
    bits.skip(1);
    let [crop_left, crop_right, crop_top, crop_bottom] = if bits.read_flag() {
        [0; 4].map(|_| bits.read_exp_golomb())
    } else {
        [0; 4]
    };
    let field_factor = 2 - u32::from(is_frame_only);
    let (crop_unit_x, crop_unit_y) = match chroma_format {
        1 => (2, 2 * field_factor),
        2 => (2, field_factor),
        _ => (1, field_factor),
    };
    let width = width_in_macroblocks * 16 - crop_unit_x * (crop_left + crop_right);
    let height = field_factor * height_in_map_units * 16 - crop_unit_y * (crop_top + crop_bottom);
    (width, height)
}

fn skip_scaling_list(bits: &mut BitReader, size: usize) {
    let mut last_scale = 8;
    let mut next_scale = 8;
    for _ in 0..size {
        if next_scale != 0 {
            next_scale = (last_scale + bits.read_signed_exp_golomb() + 256) % 256;
        }
        if next_scale != 0 {
            last_scale = next_scale;
        }
    }
}

fn remove_emulation_prevention(nal: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(nal.len());
    let mut zero_count = 0;
    for &byte in nal {
        if zero_count >= 2 && byte == 3 {
            zero_count = 0;
            continue;
        }
        zero_count = if byte == 0 { zero_count + 1 } else { 0 };
        payload.push(byte);
    }
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyframe(sequence_parameter_set: &[u8], picture_parameter_set: &[u8]) -> Vec<u8> {
        [
            &[0, 0, 0, 1][..],
            sequence_parameter_set,
            &[0, 0, 0, 1],
            picture_parameter_set,
            &[0, 0, 1, 0x65, 0x88],
        ]
        .concat()
    }

    #[test]
    fn reads_the_cropped_size_of_a_videotoolbox_1080p_keyframe() {
        let sequence_parameter_set = [
            0x27, 0x64, 0x00, 0x28, 0xAC, 0x56, 0x80, 0x78, 0x02, 0x27, 0xE5, 0x40,
        ];
        let picture_parameter_set = [0x28, 0xEE, 0x3C, 0xB0];
        let Some(VideoFormat::H264 {
            width,
            height,
            sequence_parameter_set: sps,
            picture_parameter_set: pps,
        }) = video_format(&keyframe(&sequence_parameter_set, &picture_parameter_set))
        else {
            panic!("no H.264 format");
        };
        assert_eq!((width, height), (1920, 1080));
        assert_eq!(
            (sps.as_slice(), pps.as_slice()),
            (&sequence_parameter_set[..], &picture_parameter_set[..])
        );
    }

    #[test]
    fn reads_the_size_of_an_x264_sequence_with_emulation_prevention() {
        let sequence_parameter_set = [
            0x67, 0x64, 0x00, 0x2A, 0xAC, 0xD9, 0x40, 0x56, 0x06, 0x1E, 0x6F, 0x01, 0x10, 0x00,
            0x00, 0x03, 0x00, 0x10, 0x00, 0x00, 0x07, 0x80, 0xF1, 0x83, 0x19, 0x60,
        ];
        assert_eq!(read_size(&sequence_parameter_set), (1366, 768));
    }

    #[test]
    fn prefixes_each_nal_unit_with_its_length() {
        let annex_b = [0, 0, 0, 1, 0x67, 0xAA, 0, 0, 1, 0x68, 0xBB, 0xCC];
        assert_eq!(
            length_prefixed(&annex_b),
            [0, 0, 0, 2, 0x67, 0xAA, 0, 0, 0, 3, 0x68, 0xBB, 0xCC]
        );
    }
}
