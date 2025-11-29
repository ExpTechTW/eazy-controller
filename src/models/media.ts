export interface MediaInfo {
  session_id: string; 
  app_name: string;
  title: string;
  artist: string;
  album: string;
  is_playing: boolean;
  thumbnail: string | null;
  can_go_next: boolean;   
  can_go_previous: boolean;  
}

