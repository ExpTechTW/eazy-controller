'use client';

import { Volume2, VolumeX } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { AudioSession } from '@/models/home';

interface SessionListProps {
  sessions: AudioSession[];
  onVolumeChange: (sessionName: string, volume: number) => void;
  onMuteToggle: (sessionName: string, currentMuted: boolean) => void;
}

export function SessionList({
  sessions,
  onVolumeChange,
  onMuteToggle,
}: SessionListProps) {
  if (sessions.length === 0) {
    return (
      <div className="text-center py-12">
        找不到符合條件的程式
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {sessions.map((session, index) => (
        <Card
          key={`${session.name}-${index}`}
          className="rounded-lg p-6 shadow-lg gap-0"
        >
          <div className="mb-4 flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold truncate flex-1 min-w-0" title={session.name}>
              {session.name}
            </h2>
            <div className="flex items-center gap-2">
              {session.is_muted ? (
                <VolumeX className="w-4 h-4 text-destructive shrink-0" />
              ) : (
                <Volume2 className="w-4 h-4 text-green-600 shrink-0" />
              )}
              <Switch
                checked={!session.is_muted}
                onCheckedChange={() => onMuteToggle(session.name, session.is_muted)}
              />
            </div>
          </div>

          <div>
            <div className="flex justify-between text-sm mb-2">
              <span>音量</span>
              <span>{Math.round(session.volume * 100)}%</span>
            </div>
            <Slider
              value={[Math.round(session.volume * 100)]}
              onValueChange={(value) => onVolumeChange(session.name, value[0])}
              min={0}
              max={100}
              step={1}
              className="w-full"
            />
          </div>
        </Card>
      ))}
    </div>
  );
}
