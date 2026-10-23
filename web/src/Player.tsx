import { useState, type ComponentProps } from "react";
import {
  AdamoProvider,
  AudioStream,
  GamepadController,
  Stream,
  useOrg,
  useRobotOnline,
  useSubscription,
  type StreamStats,
} from "adamo-react";
import { decodeRumbleMessage, playRumble, rumbleKey } from "adamo/teleop";
import { PausedOverlay } from "./PausedOverlay";
import { useInputCapture, usePointerLocked } from "./useInputCapture";

const GAMEPAD_INDICES = [0, 1, 2, 3];

type PlayerProps = {
  robot: string;
  audioSession: ComponentProps<typeof AdamoProvider>["session"];
  onExit: () => void;
};

export function Player({ robot, audioSession, onExit }: PlayerProps) {
  const org = useOrg() ?? "";
  const isOnline = useRobotOnline(robot);
  const isLocked = usePointerLocked();
  const [stats, setStats] = useState<StreamStats | null>(null);

  useInputCapture(org, robot);
  useSubscription(org ? rumbleKey(org, robot) : "", (sample) => {
    playRumble(decodeRumbleMessage(sample.payload().toBytes()));
  });

  return (
    <div className="fixed inset-0 bg-black">
      <Stream
        robot={robot}
        track="screen"
        videoCodec="av1"
        jitterMaxMs={0}
        robotOnline={isOnline}
        onStats={setStats}
        style={{ width: "100%", height: "100%" }}
      />
      <AdamoProvider session={audioSession} org={org}>
        <AudioStream robot={robot} track="audio" />
      </AdamoProvider>
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
