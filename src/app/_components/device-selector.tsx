'use client';

import { Volume2, VolumeX } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { AudioDevice } from '@/models/home';

interface DeviceSelectorProps {
  devices: AudioDevice[];
  defaultDeviceVolume: number;
  defaultDeviceMuted: boolean;
  onSetDefaultDevice: (deviceId: string) => void;
  onVolumeChange: (volume: number) => void;
  onMuteToggle: () => void;
}

export function DeviceSelector({
  devices,
  defaultDeviceVolume,
  defaultDeviceMuted,
  onSetDefaultDevice,
  onVolumeChange,
  onMuteToggle,
}: DeviceSelectorProps) {
  return (
    <Card className="mb-8 rounded-lg p-6 gap-0">
      <label className="text-xl font-semibold mb-4 flex items-center gap-2">
        <Volume2 className="w-5 h-5" />
        默認音源輸出
      </label>
      {devices.length === 0 ? (
        <div>目前沒有檢測到音源輸出設備</div>
      ) : (
        <Select
          value={devices.find(d => d.is_default)?.id || ''}
          onValueChange={onSetDefaultDevice}
        >
          <SelectTrigger className="w-full">
            <SelectValue placeholder="選擇輸出設備" />
          </SelectTrigger>
          <SelectContent>
            {devices.map((device) => (
              <SelectItem key={device.id} value={device.id}>
                {device.name} {device.is_default ? '(默認)' : ''}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
      <div className="mt-6 space-y-4">
        <div>
          <div className="flex justify-between text-sm mb-3">
            <span className="font-medium">音量</span>
            <span className="text-lg font-bold">{defaultDeviceVolume}%</span>
          </div>
          <Slider
            value={[defaultDeviceVolume]}
            onValueChange={(value) => onVolumeChange(value[0])}
            min={0}
            max={100}
            step={1}
            className="w-full"
          />
        </div>

        <div className="flex items-center justify-between p-4 rounded-lg border bg-card">
          <div className="flex items-center gap-3 flex-1 min-w-0">
            {defaultDeviceMuted ? (
              <VolumeX className="w-5 h-5 text-destructive shrink-0" />
            ) : (
              <Volume2 className="w-5 h-5 text-green-600 shrink-0" />
            )}
            <span className="font-medium truncate">
              {defaultDeviceMuted ? '系统已靜音' : '系统音量正常'}
            </span>
          </div>
          <Switch
            checked={!defaultDeviceMuted}
            onCheckedChange={onMuteToggle}
          />
        </div>
      </div>
    </Card>
  );
}
