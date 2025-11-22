'use client';

import { Music, Play, Pause, SkipForward, SkipBack } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { MediaInfo } from '@/models/media';

interface MediaPlayerProps {
  mediaInfo: MediaInfo | null;
  allMediaSessions: MediaInfo[];
  selectedSessionId: string;
  onSessionSelect: (sessionId: string) => void;
  onPlayPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
}

export function MediaPlayer({
  mediaInfo,
  allMediaSessions,
  selectedSessionId,
  onSessionSelect,
  onPlayPause,
  onNext,
  onPrevious,
}: MediaPlayerProps) {
  if (!mediaInfo) return null;

  return (
    <Card className="mb-8 rounded-lg p-6 gap-0">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Music className="w-5 h-5" />
          正在播放
        </h2>
        {mediaInfo.is_playing ? (
          <Play className="w-5 h-5 text-green-600" />
        ) : (
          <Pause className="w-5 h-5 text-gray-400" />
        )}
      </div>

      {allMediaSessions.length > 1 && (
        <div className="mb-4">
          <label className="text-sm font-medium mb-2 block">選擇媒體來源</label>
          <Select value={selectedSessionId} onValueChange={onSessionSelect}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="選擇播放裝置" />
            </SelectTrigger>
            <SelectContent>
              {allMediaSessions.map((session) => (
                <SelectItem key={session.session_id} value={session.session_id}>
                  <div className="flex items-center gap-2">
                    {session.is_playing ? (
                      <Play className="w-3 h-3 text-green-600 shrink-0" />
                    ) : (
                      <Pause className="w-3 h-3 text-gray-400 shrink-0" />
                    )}
                    <span>{session.app_name}</span>
                    {session.title && (
                      <span className="text-xs truncate max-w-[200px]">
                        - {session.title}
                      </span>
                    )}
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}

      <div className="flex gap-4">
        {mediaInfo.thumbnail ? (
          <div className="shrink-0">
            <img
              src={`data:image/png;base64,${mediaInfo.thumbnail}`}
              alt="封面"
              className="w-32 h-32 rounded-lg object-cover shadow-lg"
            />
          </div>
        ) : (
          <div className="shrink-0 w-32 h-32 rounded-lg bg-gray-700 flex items-center justify-center shadow-lg">
            <Music className="w-12 h-12 text-gray-500" />
          </div>
        )}

        <div className="flex-1 min-w-0 space-y-2">
          <div className="flex justify-between items-center">
            <span>應用</span>
            <span className="font-medium">{mediaInfo.app_name || '未知'}</span>
          </div>
          <div className="flex justify-between items-center">
            <span>標題</span>
            <span className="font-medium truncate ml-4" title={mediaInfo.title}>
              {mediaInfo.title || '無標題'}
            </span>
          </div>
          {mediaInfo.artist && (
            <div className="flex justify-between items-center">
              <span>藝術家</span>
              <span className="font-medium truncate ml-4" title={mediaInfo.artist}>
                {mediaInfo.artist}
              </span>
            </div>
          )}
          {mediaInfo.album && (
            <div className="flex justify-between items-center">
              <span>專輯</span>
              <span className="font-medium truncate ml-4" title={mediaInfo.album}>
                {mediaInfo.album}
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="flex items-center justify-center gap-3 mt-6 pt-4 border-t">
        <Button
          onClick={onPrevious}
          variant="outline"
          size="lg"
          className={`rounded-full ${!mediaInfo.can_go_previous ? 'opacity-40' : ''}`}
          title={!mediaInfo.can_go_previous ? '當前播放器不支援上一首，點擊嘗試切換到其他播放器' : '上一首'}
        >
          <SkipBack className="w-5 h-5" />
        </Button>

        <Button
          onClick={onPlayPause}
          variant="default"
          size="lg"
          className="rounded-full w-14 h-14 bg-blue-600 hover:bg-blue-700 shadow-lg shadow-blue-500/50"
        >
          {mediaInfo.is_playing ? (
            <Pause className="w-6 h-6" />
          ) : (
            <Play className="w-6 h-6 ml-0.5" />
          )}
        </Button>

        <Button
          onClick={onNext}
          variant="outline"
          size="lg"
          className={`rounded-full ${!mediaInfo.can_go_next ? 'opacity-40' : ''}`}
          title={!mediaInfo.can_go_next ? '當前播放器不支援下一首，點擊嘗試切換到其他播放器' : '下一首'}
        >
          <SkipForward className="w-5 h-5" />
        </Button>
      </div>
    </Card>
  );
}
