import { useEffect, useState, type ComponentProps } from "react";
import {
  AdamoProvider,
  AudioStream,
  GamepadController,
  Reliability,
  Stream,
  useOrg,
  usePublisher,
  useRobotOnline,
  useSubscription,
  type StreamStats,
} from "adamo-react";
import { controlKey, decodeRumbleMessage, playRumble, rumbleKey } from "adamo/teleop";
import { PausedOverlay } from "./PausedOverlay";
import { useInputCapture, usePointerLocked } from "./useInputCapture";

const GAMEPAD_INDICES = [0, 1, 2, 3];

type PlayerProps = {
  robot: string;
  audioSession?: ComponentProps<typeof AdamoProvider>["session"];
  onExit: () => void;
};

export function Player({ robot, audioSession, onExit }: PlayerProps) {
  const org = useOrg() ?? "";
  const isOnline = useRobotOnline(robot);
  const isLocked = usePointerLocked();
  const [stats, setStats] = useState<StreamStats | null>(null);
  const display = usePublisher(org ? controlKey(org, robot, "json/display") : "", {
    reliability: Reliability.RELIABLE,
  });

  useEffect(() => {
    if (!display) return;
    const width = Math.round(Math.max(screen.width, screen.height) * devicePixelRatio);
    const height = Math.round(Math.min(screen.width, screen.height) * devicePixelRatio);
    void display.put(new TextEncoder().encode(JSON.stringify({ width, height })));
  }, [display]);

  useInputCapture(org, robot);
  useSubscription(org ? rumbleKey(org, robot) : "", (sample) => {
    playRumble(decodeRumbleMessage(sample.payload().toBytes()));
  });

  return (
    <div className="fixed inset-0 bg-black">
      <Stream
        robot={robot}
        track="screen"
        jitterMaxMs={0}
        robotOnline={isOnline}
        onStats={setStats}
        style={{ width: "100%", height: "100%" }}
      />
      {audioSession ? (
        <AdamoProvider session={audioSession} org={org}>
          <AudioStream robot={robot} track="audio" />
        </AdamoProvider>
      ) : (
        <AudioStream robot={robot} track="audio" />
      )}
      {org &&
        GAMEPAD_INDICES.map((index) => (
          <GamepadController
            key={index}
            topic={`joy/${index}`}
            config={{ org, robotName: robot, deviceId: index, serializationFormat: "msgpack" }}
          />
        ))}
      {!isLocked && (
        <PausedOverlay robot={robot} isOnline={isOnline} stats={stats} onExit={onExit} />
      )}
    </div>
  );
}
