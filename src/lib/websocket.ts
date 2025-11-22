export type WebSocketMessage = {
  type: string;
  data?: unknown;
  message?: string;
};

export type EventCallback<T> = (data: T) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectInterval: number = 3000;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 10;
  private eventListeners: Map<string, Set<EventCallback<unknown>>> = new Map();
  private pendingRequests: Map<string, { resolve: (value: unknown) => void; reject: (error: unknown) => void }> = new Map();
  private requestId: number = 0;
  private isIntentionallyClosed: boolean = false;

  constructor(url: string) {
    this.url = url;
  }

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url);
        this.isIntentionallyClosed = false;

        this.ws.onopen = () => {
          console.log('WebSocket 已連接:', this.url);
          this.reconnectAttempts = 0;
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data);
            this.handleMessage(message);
          } catch (error) {
            console.error('無法解析 WebSocket 訊息:', error);
          }
        };

        this.ws.onerror = (error) => {
          console.error('WebSocket 錯誤:', error);
          reject(error);
        };

        this.ws.onclose = () => {
          console.log('WebSocket 已斷線');
          if (!this.isIntentionallyClosed) {
            this.attemptReconnect();
          }
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`嘗試重新連接 (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      setTimeout(() => {
        this.connect().catch((error) => {
          console.error('重新連接失敗:', error);
        });
      }, this.reconnectInterval);
    } else {
      console.error('已達到最大重連次數');
      this.emitEvent('connection_lost', null);
    }
  }

  private handleMessage(message: WebSocketMessage) {
    if (message.type === 'media_info_updated' ||
        message.type === 'media_thumbnail_updated' ||
        message.type === 'media_info_cleared') {
      this.emitEvent(message.type, message.data);
      return;
    }

    this.emitEvent(message.type, message.data || message.message);
  }

  private emitEvent(eventType: string, data: unknown) {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach((callback) => callback(data));
    }
  }

  send(type: string, data?: unknown): Promise<unknown> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('WebSocket 未連接'));
        return;
      }

      const message = { type, data };

      try {
        this.ws.send(JSON.stringify(message));

        const responseType = this.getResponseType(type);
        const cleanup = this.on(responseType, (responseData: unknown) => {
          cleanup();
          resolve(responseData);
        });

        const errorCleanup = this.on('error', (error: unknown) => {
          errorCleanup();
          cleanup();
          reject(new Error(error as string));
        });

        setTimeout(() => {
          cleanup();
          errorCleanup();
          reject(new Error('請求超時'));
        }, 10000);
      } catch (error) {
        reject(error);
      }
    });
  }

  private getResponseType(requestType: string): string {
    const typeMap: Record<string, string> = {
      'get_audio_sessions': 'audio_sessions',
      'get_audio_devices': 'audio_devices',
      'get_default_device_volume': 'default_device_volume',
      'get_default_device_mute': 'default_device_mute',
      'get_all_media_sessions': 'all_media_sessions',
      'get_media_info': 'media_info',
      'get_media_thumbnail': 'media_thumbnail',
      'set_session_volume': 'success',
      'set_session_mute': 'success',
      'set_default_device': 'success',
      'set_default_device_volume': 'success',
      'set_default_device_mute': 'success',
      'media_play_pause': 'success',
      'media_next': 'success',
      'media_previous': 'success',
    };
    return typeMap[requestType] || 'success';
  }

  on(eventType: string, callback: EventCallback<unknown>): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(callback);

    return () => {
      const listeners = this.eventListeners.get(eventType);
      if (listeners) {
        listeners.delete(callback);
      }
    };
  }

  disconnect() {
    this.isIntentionallyClosed = true;
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}
