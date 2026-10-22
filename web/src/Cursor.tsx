import { useRef } from "react";
import { useSubscription } from "adamo-react";

type CursorImage = { width: number; height: number; hot_x: number; hot_y: number; rgba: string };
type CursorPosition = { x: number; y: number; width: number; height: number };

const decoder = new TextDecoder();

export function Cursor({ org, robot }: { org: string; robot: string }) {
  const image = useRef<HTMLImageElement>(null);
  const hotspot = useRef({ x: 0, y: 0 });
  const lastImage = useRef("");

  useSubscription(org ? `adamo/${org}/${robot}/cursor/image` : "", (sample) => {
    const cursor: CursorImage = JSON.parse(decoder.decode(sample.payload().toBytes()));
    hotspot.current = { x: cursor.hot_x, y: cursor.hot_y };
    if (cursor.rgba === lastImage.current || !image.current || cursor.width === 0) return;
    lastImage.current = cursor.rgba;
    const canvas = document.createElement("canvas");
    canvas.width = cursor.width;
    canvas.height = cursor.height;
    canvas.getContext("2d")!.putImageData(new ImageData(Uint8ClampedArray.from(atob(cursor.rgba), (character) => character.charCodeAt(0)), cursor.width, cursor.height), 0, 0);
    image.current.src = canvas.toDataURL();
  });

  useSubscription(org ? `adamo/${org}/${robot}/cursor/position` : "", (sample) => {
    if (!image.current) return;
    const position: CursorPosition = JSON.parse(decoder.decode(sample.payload().toBytes()));
    const scale = Math.min(window.innerWidth / position.width, window.innerHeight / position.height);
    const left = (window.innerWidth - position.width * scale) / 2 + (position.x - hotspot.current.x) * scale;
    const top = (window.innerHeight - position.height * scale) / 2 + (position.y - hotspot.current.y) * scale;
    image.current.style.transform = `translate(${left}px, ${top}px) scale(${scale})`;
  });

  return <img ref={image} alt="" className="pointer-events-none absolute left-0 top-0 origin-top-left" />;
}
