'use client';

import { useEffect, useState, useRef } from 'react';
import { audioController } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { RefreshCcw } from 'lucide-react';
import { AudioSession, AudioDevice } from '@/models/home';
import { MediaInfo } from '@/models/media';
import { MediaPlayer } from './_components/media-player';
import { DeviceSelector } from './_components/device-selector';
import { SearchBar } from './_components/search-bar';
import { SessionList } from './_components/session-list';


export default function Home() {
  const [sessions, setSessions] = useState<AudioSession[]>([]);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [error, setError] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [defaultDeviceVolume, setDefaultDeviceVolume] = useState<number>(100);
  const [defaultDeviceMuted, setDefaultDeviceMuted] = useState<boolean>(false);
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null);
  const [allMediaSessions, setAllMediaSessions] = useState<MediaInfo[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>('');
  const selectedSessionIdRef = useRef<string>('');
  const lastThumbnailSessionId = useRef<string>('');
  const lastMediaSessionsData = useRef<string>('');
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const [searchQuery, setSearchQuery] = useState<string>('');
  const [filterType, setFilterType] = useState<'all' | 'active' | 'muted'>('all');

  const loadSessions = async () => {
    try {
      const result = await audioController.getAudioSessions();
      setSessions(result);
      setError('');
    } catch (err) {
      setError(err as string);
      console.error('Failed to load audio sessions:', err);
    }
  };

  const loadDevices = async () => {
    try {
      const result = await audioController.getAudioDevices();
      setDevices(result);
    } catch (err) {
      console.error('Failed to load audio devices:', err);
    }
  };

  const loadDefaultDeviceVolume = async () => {
    try {
      const volume = await audioController.getDefaultDeviceVolume();
      setDefaultDeviceVolume(Math.round(volume * 100));

      const muted = await audioController.getDefaultDeviceMute();
      setDefaultDeviceMuted(muted);
    } catch (err) {
      console.error('Failed to load default device volume:', err);
    }
  };

  const loadAllMediaSessions = async () => {
    try {
      const result = await audioController.getAllMediaSessions();

      const sessionListForCompare = result.map(s => ({
        session_id: s.session_id,
        app_name: s.app_name
      }));
      const sessionListString = JSON.stringify(sessionListForCompare);

      setAllMediaSessions(result);

      const sessionListChanged = lastMediaSessionsData.current !== sessionListString;
      if (sessionListChanged) {
        lastMediaSessionsData.current = sessionListString;
      }

      if (!selectedSessionIdRef.current && result.length > 0) {
        const firstSession = result[0];
        selectedSessionIdRef.current = firstSession.session_id;
        setSelectedSessionId(firstSession.session_id);
        setMediaInfo(firstSession);
        lastThumbnailSessionId.current = firstSession.session_id;
        loadMediaThumbnail(firstSession.session_id);
        return; 
      }

      if (selectedSessionIdRef.current && sessionListChanged) {
        const stillExists = result.find(s => s.session_id === selectedSessionIdRef.current);
        if (!stillExists) {
          if (result.length > 0) {
            const firstSession = result[0];
            selectedSessionIdRef.current = firstSession.session_id;
            setSelectedSessionId(firstSession.session_id);
            setMediaInfo(firstSession);
            lastThumbnailSessionId.current = firstSession.session_id;
            loadMediaThumbnail(firstSession.session_id);
          } else {
            setMediaInfo(null);
            selectedSessionIdRef.current = '';
            setSelectedSessionId('');
            lastThumbnailSessionId.current = '';
          }
          return; 
        }
      }

      if (selectedSessionIdRef.current) {
        const selected = result.find(s => s.session_id === selectedSessionIdRef.current);

        if (selected) {
          setMediaInfo(prev => {
            if (!prev) {
              if (selected.title || selected.artist || selected.album) {
                loadMediaThumbnail(selectedSessionId);
              }
              return selected;
            }

            const titleChanged = prev.title !== selected.title;
            const artistChanged = prev.artist !== selected.artist;
            const albumChanged = prev.album !== selected.album;
            const isPlayingChanged = prev.is_playing !== selected.is_playing;
            const appNameChanged = prev.app_name !== selected.app_name;
            const canGoNextChanged = prev.can_go_next !== selected.can_go_next;
            const canGoPreviousChanged = prev.can_go_previous !== selected.can_go_previous;

            const hasChanged = titleChanged || artistChanged || albumChanged || isPlayingChanged || appNameChanged || canGoNextChanged || canGoPreviousChanged;

            if (!hasChanged) {
              return prev;
            }

            if (titleChanged || artistChanged || albumChanged) {
              loadMediaThumbnail(selectedSessionId);
              return { ...selected, thumbnail: prev.thumbnail };
            }

            return { ...selected, thumbnail: prev.thumbnail };
          });
        }
      }
    } catch (err) {
      console.error('Failed to load media sessions:', err);
      setAllMediaSessions([]);
      setMediaInfo(null);
      lastThumbnailSessionId.current = '';
      lastMediaSessionsData.current = '';
    }
  };

  const loadMediaInfo = async () => {
    await loadAllMediaSessions();
  };

  const startPollingForData = () => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }

    let attemptCount = 0;
    const maxAttempts = 10;


    pollingIntervalRef.current = setInterval(async () => {
      attemptCount++;

      try {
        const result = await audioController.getAllMediaSessions();
        const selected = result.find(s => s.session_id === selectedSessionIdRef.current);

        if (selected && (selected.title || selected.artist || selected.album)) {
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }

          setAllMediaSessions(result);
          setMediaInfo(prev  => {
            loadMediaThumbnail(selectedSessionIdRef.current);
            return { ...selected, thumbnail: prev?.thumbnail as string | null };
          });
        } else if (attemptCount >= maxAttempts) {
          if (pollingIntervalRef.current) {
            clearInterval(pollingIntervalRef.current);
            pollingIntervalRef.current = null;
          }
        }
      } catch (err) {
        console.error('輪詢失敗:', err);
      }
    }, 1000);
  };

  const loadMediaThumbnail = async (sessionId?: string) => {
    try {
      const targetSessionId = sessionId || selectedSessionIdRef.current;
      if (!targetSessionId) return;

      const thumbnail = await audioController.getMediaThumbnail(targetSessionId);

      if (thumbnail) {
        setMediaInfo(prev => {
          if (prev && prev.session_id === targetSessionId) {
            return { ...prev, thumbnail };
          }
          return prev;
        });
      }
    } catch (err) {
      console.error('Failed to load thumbnail:', err);
    }
  };

  const handleMediaPlayPause = () => {
    audioController.mediaPlayPause(selectedSessionIdRef.current || undefined)
      .then(() => {
        setTimeout(() => startPollingForData(), 300);
      })
      .catch((err) => {
        console.error('Failed to toggle media playback:', err);
      });
  };

  const handleMediaNext = () => {
    if (mediaInfo && !mediaInfo.can_go_next && allMediaSessions.length > 1) {
      const nextSession = allMediaSessions.find(s => s.can_go_next && s.session_id !== mediaInfo.session_id);
      if (nextSession) {
        handleSessionSelect(nextSession.session_id);
        setTimeout(() => {
          audioController.mediaNext(nextSession.session_id)
            .then(() => {
              setTimeout(() => startPollingForData(), 300);
            })
            .catch((err) => {
              console.error('Failed to skip to next track:', err);
            });
        }, 100);
        return;
      }
    }

    audioController.mediaNext(selectedSessionIdRef.current || undefined)
      .then(() => {
        setTimeout(() => startPollingForData(), 300);
      })
      .catch((err) => {
        console.error('Failed to skip to next track:', err);
      });
  };

  const handleMediaPrevious = () => {
    if (mediaInfo && !mediaInfo.can_go_previous && allMediaSessions.length > 1) {
      const prevSession = allMediaSessions.find(s => s.can_go_previous && s.session_id !== mediaInfo.session_id);
      if (prevSession) {
        handleSessionSelect(prevSession.session_id);
        setTimeout(() => {
          audioController.mediaPrevious(prevSession.session_id)
            .then(() => {
              setTimeout(() => startPollingForData(), 300);
            })
            .catch((err) => {
              console.error('Failed to skip to previous track:', err);
            });
        }, 100);
        return;
      }
    }

    audioController.mediaPrevious(selectedSessionIdRef.current || undefined)
      .then(() => {
        setTimeout(() => startPollingForData(), 300);
      })
      .catch((err) => {
        console.error('Failed to skip to previous track:', err);
      });
  };

  const handleSetDefaultDevice = async (deviceId: string) => {
    try {
      await audioController.setDefaultDevice(deviceId);
      await loadDevices();
    } catch (err) {
      console.error('Failed to set default device:', err);
      alert(`設定默認設備失敗: ${err}`);
    }
  };

  const handleVolumeChange = async (sessionName: string, volume: number) => {
    try {
      await audioController.setSessionVolume(sessionName, volume / 100);
      setSessions(sessions.map(s =>
        s.name === sessionName ? { ...s, volume: volume / 100 } : s
      ));
    } catch (err) {
      console.error('無法設定音量:', err);
    }
  };

  const handleMuteToggle = async (sessionName: string, currentMuted: boolean) => {
    try {
      await audioController.setSessionMute(sessionName, !currentMuted);
        setSessions(sessions.map(s =>
          s.name === sessionName ? { ...s, is_muted: !currentMuted } : s
        ));
    } catch (err) {
      console.error('無法切換靜音:', err);
    }
  };

  const handleDefaultDeviceVolumeChange = async (volume: number) => {
    try {
      await audioController.setDefaultDeviceVolume(volume / 100);
      setDefaultDeviceVolume(volume);
    } catch (err) {
      console.error('無法設定默認設備音量:', err);
    }
  };

  const handleDefaultDeviceMuteToggle = async () => {
    try {
      const newMuted = !defaultDeviceMuted;
      await audioController.setDefaultDeviceMute(newMuted);
      setDefaultDeviceMuted(newMuted);
    } catch (err) {
      console.error('無法切換默認設備靜音:', err);
    }
  };

  const handleSessionSelect = (sessionId: string) => {
    selectedSessionIdRef.current = sessionId;
    setSelectedSessionId(sessionId);
    const selected = allMediaSessions.find(s => s.session_id === sessionId);
    if (selected) {
      setMediaInfo(selected);
      lastThumbnailSessionId.current = sessionId;
      loadMediaThumbnail(sessionId);
    }
  };

  const filteredSessions = sessions
    .filter(session => {
      const matchesSearch = session.name.toLowerCase().includes(searchQuery.toLowerCase());

      let matchesFilter = true;
      if (filterType === 'active') {
        matchesFilter = !session.is_muted && session.volume > 0;
      } else if (filterType === 'muted') {
        matchesFilter = session.is_muted || session.volume === 0;
      }

      return matchesSearch && matchesFilter;
    })
    .sort((a, b) => {
      return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
    });

  useEffect(() => {
    const initConnection = async () => {
      if (audioController.getConnectionMode() === 'websocket' && !audioController.isConnected()) {
        try {
          const urlsToTry = 'ws://eazycontroller.local/ws'

          let connected = false;
          const lastError: Error | null = null;

          await audioController.initWebSocket(urlsToTry);
          connected = true;

          if (!connected) {
            throw lastError || new Error('無法連接到任何 WebSocket 伺服器');
          }
        } catch (err) {
          console.error('WebSocket 連接失敗:', err);
          setError('無法連接到 Eazy Controller，請確認應用程式已啟動');
          setLoading(false);
          return;
        }
      }

      try {
        setLoading(true);

        await Promise.race([
          Promise.all([
            loadSessions(),
            loadDevices(),
            loadDefaultDeviceVolume(),
            loadMediaInfo(),
          ]),
          new Promise((_, reject) =>
            setTimeout(() => {
              console.error('[頁面] ✗ 載入超時（10秒）');
              reject(new Error('加載超時'));
            }, 10000)
          )
        ]);

      } catch (err) {
        console.error('[頁面] 加載數據失敗:', err);
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };

    initConnection();

    const mediaRefreshInterval = setInterval(() => {
      loadAllMediaSessions();
    }, 2000);

    let cleanupEventListeners: (() => void) | null = null;

    const setupEventListeners = async () => {
      const unlistenMediaInfo = await audioController.onMediaInfoUpdated((mediaInfo) => {
        if (selectedSessionId && mediaInfo.session_id === selectedSessionId) {
          setMediaInfo(mediaInfo);
        }

        setAllMediaSessions(prev => {
          const index = prev.findIndex(s => s.session_id === mediaInfo.session_id);
          if (index !== -1) {
            const newSessions = [...prev];
            newSessions[index] = mediaInfo;
            return newSessions;
          }
          return prev;
        });
      });

      const unlistenThumbnail = await audioController.onMediaThumbnailUpdated(() => {
        loadAllMediaSessions();
      });

      const unlistenClear = await audioController.onMediaInfoCleared(() => {
        setMediaInfo(null);
      });

      cleanupEventListeners = () => {
        unlistenMediaInfo();
        unlistenThumbnail();
        unlistenClear();
      };
    };

    setupEventListeners();

    return () => {
      clearInterval(mediaRefreshInterval);
      if (cleanupEventListeners) {
        cleanupEventListeners();
      }
    };
  }, []);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <div className="text-xl">加載中...</div>
          <div className="text-sm mt-2">正在連接</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="text-xl text-red-500 mb-4">出現錯誤</div>
          <div>{error}</div>
          <Button
            onClick={() => window.location.reload()}
            variant="default"
            size="default"
            className="mt-4"
          >
            重新載入
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen p-8">
      <div className="max-w-6xl mx-auto">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold">Eazy Controller</h1>
          <Button
            onClick={async () => {
              try {
                if (audioController.getConnectionMode() === 'websocket') {
                  await audioController.reconnectWebSocket();
                }

                await Promise.all([
                  loadSessions(),
                  loadDevices(),
                  loadDefaultDeviceVolume(),
                  loadMediaInfo(),
                ]);
              } catch (err) {
                console.error('刷新失敗:', err);
                setError(`刷新失敗: ${err}`);
              }
            }}
            size="lg"
            className="text-lg rounded-2xl shadow-lg shadow-blue-500/50"
          >
            <RefreshCcw className="w-4 h-4" />
            重新整理
          </Button>
        </div>

        <MediaPlayer
          mediaInfo={mediaInfo}
          allMediaSessions={allMediaSessions}
          selectedSessionId={selectedSessionId}
          onSessionSelect={handleSessionSelect}
          onPlayPause={handleMediaPlayPause}
          onNext={handleMediaNext}
          onPrevious={handleMediaPrevious}
        />

        <DeviceSelector
          devices={devices}
          defaultDeviceVolume={defaultDeviceVolume}
          defaultDeviceMuted={defaultDeviceMuted}
          onSetDefaultDevice={handleSetDefaultDevice}
          onVolumeChange={handleDefaultDeviceVolumeChange}
          onMuteToggle={handleDefaultDeviceMuteToggle}
        />

        <SearchBar
          searchQuery={searchQuery}
          filterType={filterType}
          sessionCounts={{
            all: sessions.length,
            active: sessions.filter(s => !s.is_muted && s.volume > 0).length,
            muted: sessions.filter(s => s.is_muted || s.volume === 0).length,
          }}
          onSearchChange={setSearchQuery}
          onFilterChange={setFilterType}
        />

        {sessions.length === 0 ? (
          <div className="text-center py-12">
            目前沒有偵測到程式
          </div>
        ) : filteredSessions.length === 0 ? (
          <div className="text-center py-12">
            找不到符合條件的程式
          </div>
        ) : (
          <SessionList
            sessions={filteredSessions}
            onVolumeChange={handleVolumeChange}
            onMuteToggle={handleMuteToggle}
          />
        )}
      </div>
    </div>
  );
}