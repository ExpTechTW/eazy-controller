export interface AudioSession {
    name: string;
    volume: number;
    is_muted: boolean;
  }
  
export interface AudioDevice {
    id: string;
    name: string;
    is_default: boolean;
  }