import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WebSocketClient } from './websocket';
import { AudioSession, AudioDevice } from '@/models/home';
import { MediaInfo } from '@/models/media';

export type EventCallback<T> = (data: T) => void;

class AudioControllerAPI {
  private wsClient: WebSocketClient | null = null;
  private wsUrl: string = '';

  constructor() {
  }

  private get isTauri(): boolean {
    if (typeof window === 'undefined') return false;

    if ('isTauri' in window && (window as Window & { isTauri?: boolean }).isTauri === true) {
      return true;
    }

    return '__TAURI_INTERNALS__' in window;
  }

  async initWebSocket(url: string): Promise<void> {
    if (this.isTauri) {
      console.log('在 Tauri 環境中，無需初始化 WebSocket');
      return;
    }

    this.wsUrl = url;
    this.wsClient = new WebSocketClient(url);
    await this.wsClient.connect();
    console.log('WebSocket 已初始化:', url);
  }

  /**
   * 檢查連接狀態
   */
  isConnected(): boolean {
    if (this.isTauri) {
      return true; // Tauri 總是連接的
    }
    return this.wsClient?.isConnected() || false;
  }

  /**
   * 獲取當前連接模式
   */
  getConnectionMode(): 'tauri' | 'websocket' {
    return this.isTauri ? 'tauri' : 'websocket';
  }

  async getAudioSessions(): Promise<AudioSession[]> {
    if (this.isTauri) {
      const result = await invoke<AudioSession[]>('get_audio_sessions');
      return result;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_audio_sessions') as Promise<AudioSession[]>;
    }
  }

  async setSessionVolume(sessionName: string, volume: number): Promise<void> {
    if (this.isTauri) {
      return invoke('set_session_volume', { sessionName, volume });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('set_session_volume', { session_name: sessionName, volume });
    }
  }

  async setSessionMute(sessionName: string, mute: boolean): Promise<void> {
    if (this.isTauri) {
      return invoke('set_session_mute', { sessionName, mute });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('set_session_mute', { session_name: sessionName, mute });
    }
  }

  async getAudioDevices(): Promise<AudioDevice[]> {
    if (this.isTauri) {
      const result = await invoke<AudioDevice[]>('get_audio_devices');
      return result;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_audio_devices') as Promise<AudioDevice[]>;
    }
  }

  async setDefaultDevice(deviceId: string): Promise<void> {
    if (this.isTauri) {
      return invoke('set_default_device', { deviceId });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('set_default_device', { device_id: deviceId });
    }
  }

  async getDefaultDeviceVolume(): Promise<number> {
    if (this.isTauri) {
      return invoke<number>('get_default_device_volume');
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_default_device_volume') as Promise<number>;
    }
  }

  async setDefaultDeviceVolume(volume: number): Promise<void> {
    if (this.isTauri) {
      return invoke('set_default_device_volume', { volume });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('set_default_device_volume', { volume });
    }
  }

  async getDefaultDeviceMute(): Promise<boolean> {
    if (this.isTauri) {
      return invoke<boolean>('get_default_device_mute');
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_default_device_mute') as Promise<boolean>;
    }
  }

  async setDefaultDeviceMute(mute: boolean): Promise<void> {
    if (this.isTauri) {
      return invoke('set_default_device_mute', { mute });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('set_default_device_mute', { mute });
    }
  }

  async getAllMediaSessions(): Promise<MediaInfo[]> {
    if (this.isTauri) {
      const result = await invoke<MediaInfo[]>('get_all_media_sessions');
      return result;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_all_media_sessions') as Promise<MediaInfo[]>;
    }
  }

  async getMediaInfo(): Promise<MediaInfo | null> {
    if (this.isTauri) {
      return invoke<MediaInfo | null>('get_media_info');
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_media_info') as Promise<MediaInfo | null>;
    }
  }

  async getMediaThumbnail(sessionId?: string): Promise<string | null> {
    if (this.isTauri) {
      return invoke<string | null>('get_media_thumbnail', { sessionId: sessionId || null });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_media_thumbnail', { session_id: sessionId || null }) as Promise<string | null>;
    }
  }

  async mediaPlayPause(sessionId?: string): Promise<void> {
    if (this.isTauri) {
      return invoke('media_play_pause', { sessionId: sessionId || null });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('media_play_pause', { session_id: sessionId || null });
    }
  }

  async mediaNext(sessionId?: string): Promise<void> {
    if (this.isTauri) {
      return invoke('media_next', { sessionId: sessionId || null });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('media_next', { session_id: sessionId || null });
    }
  }

  async mediaPrevious(sessionId?: string): Promise<void> {
    if (this.isTauri) {
      return invoke('media_previous', { sessionId: sessionId || null });
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      await this.wsClient.send('media_previous', { session_id: sessionId || null });
    }
  }

  async onMediaInfoUpdated(callback: EventCallback<MediaInfo>): Promise<() => void> {
    if (this.isTauri) {
      const unlisten = await listen<MediaInfo>('media-info-updated', (event) => {
        callback(event.payload);
      });
      return unlisten;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.on('media_info_updated', callback as EventCallback<unknown>) as () => void;
    }
  }

  async onMediaThumbnailUpdated(callback: EventCallback<string>): Promise<() => void> {
    if (this.isTauri) {
      const unlisten = await listen<string>('media-thumbnail-updated', (event) => {
        callback(event.payload);
      });
      return unlisten;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.on('media_thumbnail_updated', callback as EventCallback<unknown>) as () => void;
    }
  }

  async onMediaInfoCleared(callback: EventCallback<void>): Promise<() => void> {
    if (this.isTauri) {
      const unlisten = await listen('media-info-cleared', () => {
        callback();
      });
      return unlisten;
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.on('media_info_cleared', callback as EventCallback<unknown>) as () => void;
    }
  }

  disconnect() {
    if (this.wsClient) {
      this.wsClient.disconnect();
      this.wsClient = null;
    }
  }

  /**
   * 重新連接 WebSocket
   */
  async reconnectWebSocket(): Promise<void> {
    if (this.isTauri) {
      console.log('在 Tauri 環境中，無需重新連接 WebSocket');
      return;
    }

    // 先斷開現有連接
    this.disconnect();

    // 重新初始化
    if (this.wsUrl) {
      await this.initWebSocket(this.wsUrl);
      console.log('WebSocket 已重新連接');
    } else {
      throw new Error('沒有保存的 WebSocket URL');
    }
  }

  async resetMediaApi(): Promise<string> {
    if (this.isTauri) {
      return invoke<string>('reset_media_api');
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('reset_media_api') as Promise<string>;
    }
  }

  async getMediaApiStatus(): Promise<string> {
    if (this.isTauri) {
      return invoke<string>('get_media_api_status');
    } else {
      if (!this.wsClient) throw new Error('WebSocket 未初始化');
      return this.wsClient.send('get_media_api_status') as Promise<string>;
    }
  }
}

export const audioController = new AudioControllerAPI();

export type { AudioControllerAPI };
