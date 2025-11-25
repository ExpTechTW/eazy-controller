'use client';

import { ThemeToggle } from '@/components/theme-toggle';
import { Github, ChevronLeft, ChevronRight, Settings, AlertTriangle, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect, useRef } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';

export default function Footer() {
  const [isExpanded, setIsExpanded] = useState(false);
  const [version, setVersion] = useState('loading...');
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [hotkey, setHotkey] = useState('Alt+Z');
  const [isRecording, setIsRecording] = useState(false);
  const [keySequence, setKeySequence] = useState<string[]>([]);
  const recordingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const [savedStandardHotkey, setSavedStandardHotkey] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    let checkTauri = false;
    if (typeof window !== 'undefined') {
      const windowWithTauri = window as Window & { isTauri?: boolean; __TAURI_INTERNALS__?: unknown };
      if ('isTauri' in window && windowWithTauri.isTauri === true) {
        checkTauri = true;
      }
      else if ('__TAURI_INTERNALS__' in window) {
        checkTauri = true;
      }
    }
    setIsTauri(checkTauri);

    const fetchVersion = async () => {
      if (checkTauri) {
        const { getVersion } = await import('@tauri-apps/api/app');
        const appVersion = await getVersion();
        setVersion(`v${appVersion}`);
      } else {
        setVersion('v1.0.0-rc.3');
      }
    };

    const loadHotkey = async () => {
      if (!checkTauri) return; 
      
      const { invoke } = await import('@tauri-apps/api/core');
      const savedHotkey = await invoke<string>('load_hotkey');
      setHotkey(savedHotkey);

      const keys = savedHotkey.split('+');
      const modifiers = ['Ctrl', 'Alt', 'Shift', 'Meta'];
      const modifierKeys = keys.filter(k => modifiers.includes(k));
      const regularKeys = keys.filter(k => !modifiers.includes(k));
      const lastKey = regularKeys[regularKeys.length - 1];

      if (lastKey) {
        const modifierOrder = ['Ctrl', 'Alt', 'Shift', 'Meta'];
        const sortedModifiers = modifierKeys.sort((a, b) =>
          modifierOrder.indexOf(a) - modifierOrder.indexOf(b)
        );
        const standardHotkey = [...sortedModifiers, lastKey].join('+');
        setSavedStandardHotkey(standardHotkey);
      }
    };

    fetchVersion();
    loadHotkey();
  }, []);

  const handleGithubClick = async () => {
    if (isTauri) {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open('https://github.com/ExpTechTW/eazy-controller');
    } else {
      window.open('https://github.com/ExpTechTW/eazy-controller', '_blank');
    }
  };

  const normalizeKey = (key: string): string => {
    const keyMap: { [key: string]: string } = {
      'CONTROL': 'Ctrl',
      'ALT': 'Alt',
      'SHIFT': 'Shift',
      'META': 'Meta',
      ' ': 'Space',
      'ARROWUP': 'Up',
      'ARROWDOWN': 'Down',
      'ARROWLEFT': 'Left',
      'ARROWRIGHT': 'Right',
    };

    const upperKey = key.toUpperCase();
    return keyMap[upperKey] || upperKey;
  };

  const finishRecording = () => {
    if (keySequence.length > 0) {
      const newHotkey = keySequence.join('+');
      setHotkey(newHotkey);
    }
    setKeySequence([]);
    setIsRecording(false);

    if (recordingTimeoutRef.current) {
      clearTimeout(recordingTimeoutRef.current);
      recordingTimeoutRef.current = null;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isRecording || e.repeat) return;
    e.preventDefault();

    const normalizedKey = normalizeKey(e.key);

    setKeySequence(prev => {
      if (!prev.includes(normalizedKey)) {
        return [...prev, normalizedKey];
      }
      return prev;
    });

    if (recordingTimeoutRef.current) {
      clearTimeout(recordingTimeoutRef.current);
    }

    recordingTimeoutRef.current = setTimeout(() => {
      finishRecording();
    }, 1500);
  };


  useEffect(() => {
    if (!isTauri) return; 

    const handleDialogChange = async () => {
      const { invoke } = await import('@tauri-apps/api/core');

      if (isSettingsOpen) {
        try {
          await invoke('unregister_all_hotkeys');
        } catch (error) {
          console.error('Failed to unregister hotkeys:', error);
        }
      } else {
        if (savedStandardHotkey) {
          try {
            await invoke('register_hotkey', { hotkey: savedStandardHotkey });
          } catch (error) {
            console.error('Failed to restore hotkey:', error);
          }
        }
      }
    };

    handleDialogChange();
  }, [isSettingsOpen, savedStandardHotkey, isTauri]);

  const handleSaveHotkey = async () => {
    try {
      setError('');

      const keys = hotkey.split('+');
      const modifiers = ['Ctrl', 'Alt', 'Shift', 'Meta'];
      const modifierKeys = keys.filter(k => modifiers.includes(k));
      const regularKeys = keys.filter(k => !modifiers.includes(k));

      if (keys.length === 0) {
        setError('請至少按下一個按鍵');
        return;
      }

      if (modifierKeys.length === 0) {
        setError('全局快捷鍵必須包含至少一個修飾鍵 (Ctrl/Alt/Shift/Win)！請按鍵盤上的功能鍵,不是字母鍵。');
        return;
      }

      const lastKey = regularKeys[regularKeys.length - 1];

      if (!lastKey) {
        setError('快捷鍵必須包含至少一個普通鍵 (非 Ctrl/Alt/Shift/Meta)');
        return;
      }

      const modifierOrder = ['Ctrl', 'Alt', 'Shift', 'Meta'];
      const sortedModifiers = modifierKeys.sort((a, b) =>
        modifierOrder.indexOf(a) - modifierOrder.indexOf(b)
      );
      const standardHotkey = [...sortedModifiers, lastKey].join('+');


      setSavedStandardHotkey(standardHotkey);

      await invoke('register_hotkey', { hotkey: standardHotkey });
      await invoke('save_hotkey', { hotkey });

      setIsSettingsOpen(false);
    } catch (error) {
      console.error('Failed to register hotkey:', error);
      setError(`無法註冊快捷鍵: ${error}`);
    }
  };

  const settingCheck= () => {
    if (!isTauri) return;
    setIsSettingsOpen(!isSettingsOpen);
  }

  return (
    <>
      <footer>
        <div className="fixed bottom-4 left-4 z-50">
          <div className={`border shadow-lg border-border rounded-lg p-2 flex items-center ${isExpanded ? 'gap-2' : ''} h-11 transition-all duration-300 ease-in-out`}>
            <div className="flex items-center">
              <p className="text-xs text-muted-foreground">
                {version}
              </p>
            </div>
            <div
              className={`flex gap-1.5 transition-all duration-300 ease-in-out overflow-hidden ${
                isExpanded
                  ? 'max-w-[250px] opacity-100 translate-x-0'
                  : 'max-w-0 opacity-0 -translate-x-4'
              }`}
            >
              <div className="w-px h-auto bg-border" />
              <Button
                variant="outline"
                size="icon"
                onClick={settingCheck}
                className="rounded-lg h-7 w-7 shrink-0"
              >
                <Settings className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={handleGithubClick}
                className="rounded-lg h-7 w-7 shrink-0"
              >
                <Github className="h-4 w-4" />
              </Button>
              <div className="shrink-0">
                <ThemeToggle />
              </div>
            </div>
            {!isExpanded ? (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsExpanded(true)}
                className="rounded-lg h-5 w-5 transition-transform duration-200 hover:scale-110"
              >
                <ChevronRight className="h-3 w-3" />
              </Button>
            ) : (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsExpanded(false)}
                className="rounded-lg h-5 w-5 transition-transform duration-200 hover:scale-110"
              >
                <ChevronLeft className="h-3 w-3" />
              </Button>
            )}
          </div>
        </div>
      </footer>

      <Dialog open={isSettingsOpen} onOpenChange={settingCheck}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>設定</DialogTitle>
            <DialogDescription>
              設定全局快捷鍵來顯示/隱藏應用視窗
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">快捷鍵</label>
              <div className="flex gap-2">
                <Input
                  value={isRecording && keySequence.length > 0
                    ? keySequence.join('+')
                    : hotkey}
                  readOnly
                  onKeyDown={handleKeyDown}
                  onFocus={() => {
                    setIsRecording(true);
                    setKeySequence([]);
                    setError('');
                  }}
                  onBlur={() => {
                    finishRecording();
                  }}
                  placeholder={isRecording ? "依序按下你想要的按鍵..." : "點擊後按下快捷鍵"}
                  className={`flex-1 ${error ? 'border-red-500' : ''}`}
                />
                <Button onClick={handleSaveHotkey}>
                  儲存
                </Button>
              </div>
              {error && (
                <div className="text-sm text-red-500 flex items-start gap-2 p-2 bg-red-50 dark:bg-red-950/20 rounded-md border border-red-200 dark:border-red-900">
                  <X className="h-4 w-4 mt-0.5 shrink-0" />
                  <span>{error}</span>
                </div>
              )}
              <p className="text-xs text-muted-foreground">
                點擊輸入框，依序按下你想要的按鍵，1.5秒後自動完成設定
                <br />
                <br />
                <span className="font-semibold text-blue-500 flex items-center gap-1">
                  <AlertTriangle className="h-3 w-3" />
                  必須包含至少一個修飾鍵 (Ctrl/Alt/Shift/Win)
                </span>
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}