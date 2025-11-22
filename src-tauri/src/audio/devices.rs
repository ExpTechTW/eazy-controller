use crate::models::AudioDevice;

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::core::PWSTR;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::PropertiesSystem::*;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let default_device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("無法取得默認設備: {:?}", e))?;

            let default_id_pwstr = default_device
                .GetId()
                .map_err(|e| format!("無法取得默認設備ID: {:?}", e))?;
            let default_id = default_id_pwstr
                .to_string()
                .map_err(|e| format!("無法轉換默認ID: {:?}", e))?;

            let collection = enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let count = collection
                .GetCount()
                .map_err(|e| format!("無法獲取設備數量: {:?}", e))?;

            let mut audio_devices = Vec::new();

            for i in 0..count {
                let device = collection
                    .Item(i)
                    .map_err(|e| format!("無法獲取設備 {}: {:?}", i, e))?;

                let id_pwstr = device
                    .GetId()
                    .map_err(|e| format!("無法獲取設備ID: {:?}", e))?;
                let id = id_pwstr
                    .to_string()
                    .map_err(|e| format!("無法轉換ID: {:?}", e))?;

                let property_store = device
                    .OpenPropertyStore(STGM_READ)
                    .map_err(|e| format!("無法開啟屬性儲存: {:?}", e))?;

                let pkey = PROPERTYKEY {
                    fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
                    pid: 14,
                };

                let prop_variant = property_store
                    .GetValue(&pkey as *const _)
                    .map_err(|e| format!("無法獲取設備名稱: {:?}", e))?;

                #[repr(C)]
                struct PropVariantData {
                    vt: u16,
                    _reserved: [u16; 3],
                    data: usize,
                }

                let pv_data = std::mem::transmute::<_, &PropVariantData>(&prop_variant);
                let name = if pv_data.vt == 31 {
                    let pwstr = PWSTR(pv_data.data as *mut u16);
                    pwstr
                        .to_string()
                        .unwrap_or_else(|_| format!("Device {}", i))
                } else {
                    format!("Device {}", i)
                };

                audio_devices.push(AudioDevice {
                    id: id.clone(),
                    name,
                    is_default: id == default_id,
                });
            }

            CoUninitialize();
            Ok(audio_devices)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 設定系統預設的音訊輸出裝置
/// @param device_id 裝置的唯一識別碼
#[tauri::command]
pub fn set_default_device(device_id: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ptr;
        use windows::core::{GUID, HRESULT, PCWSTR};
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        #[repr(C)]
        struct IPolicyConfigVtbl {
            query_interface: unsafe extern "system" fn(
                *mut std::ffi::c_void,
                *const GUID,
                *mut *mut std::ffi::c_void,
            ) -> HRESULT,
            add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            get_mix_format: usize,
            get_device_format: usize,
            reset_device_format: usize,
            set_device_format: usize,
            get_processing_period: usize,
            set_processing_period: usize,
            get_share_mode: usize,
            set_share_mode: usize,
            get_property_value: usize,
            set_property_value: usize,
            set_default_endpoint:
                unsafe extern "system" fn(*mut std::ffi::c_void, PCWSTR, ERole) -> HRESULT,
            set_endpoint_visibility: usize,
        }

        #[repr(C)]
        struct IPolicyConfig {
            vtable: *const IPolicyConfigVtbl,
        }

        #[repr(C)]
        struct IPolicyConfigVistaVtbl {
            query_interface: unsafe extern "system" fn(
                *mut std::ffi::c_void,
                *const GUID,
                *mut *mut std::ffi::c_void,
            ) -> HRESULT,
            add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            get_mix_format: usize,
            get_device_format: usize,
            set_device_format: usize,
            get_processing_period: usize,
            set_processing_period: usize,
            get_share_mode: usize,
            set_share_mode: usize,
            get_property_value: usize,
            set_property_value: usize,
            set_default_endpoint:
                unsafe extern "system" fn(*mut std::ffi::c_void, PCWSTR, ERole) -> HRESULT,
        }

        #[repr(C)]
        struct IPolicyConfigVista {
            vtable: *const IPolicyConfigVistaVtbl,
        }

        const CLSID_POLICY_CONFIG_CLIENT: GUID =
            GUID::from_u128(0x870af99c_171d_4f9e_af0d_e73ae00e0e4d);
        const CLSID_POLICY_CONFIG_CLIENT_WIN7: GUID =
            GUID::from_u128(0x294935CE_F637_4E7C_A41B_AB255460B862);
        const CLSID_POLICY_CONFIG_CLIENT_WIN10: GUID =
            GUID::from_u128(0x2A07407E_6497_4A18_9706_CBFCB32D35B8);

        const IID_POLICY_CONFIG: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);
        const IID_POLICY_CONFIG_VISTA: GUID =
            GUID::from_u128(0x568b9108_44bf_40b4_9006_86afe5b5a620);

        extern "system" {
            fn CoCreateInstance(
                rclsid: *const GUID,
                punkouter: *mut std::ffi::c_void,
                dwclscontext: u32,
                riid: *const GUID,
                ppv: *mut *mut std::ffi::c_void,
            ) -> HRESULT;
        }

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let attempts = [
                (
                    "CLSID_POLICY_CONFIG_CLIENT_WIN10",
                    "IID_POLICY_CONFIG",
                    CLSID_POLICY_CONFIG_CLIENT_WIN10,
                    IID_POLICY_CONFIG,
                ),
                (
                    "CLSID_POLICY_CONFIG_CLIENT_WIN7",
                    "IID_POLICY_CONFIG",
                    CLSID_POLICY_CONFIG_CLIENT_WIN7,
                    IID_POLICY_CONFIG,
                ),
                (
                    "CLSID_POLICY_CONFIG_CLIENT",
                    "IID_POLICY_CONFIG_VISTA",
                    CLSID_POLICY_CONFIG_CLIENT,
                    IID_POLICY_CONFIG_VISTA,
                ),
                (
                    "CLSID_POLICY_CONFIG_CLIENT_WIN7",
                    "IID_POLICY_CONFIG_VISTA",
                    CLSID_POLICY_CONFIG_CLIENT_WIN7,
                    IID_POLICY_CONFIG_VISTA,
                ),
            ];

            let mut policy_config: *mut std::ffi::c_void = ptr::null_mut();
            let mut success = false;
            let mut last_error = None;
            let mut use_vista_interface = false;

            for (i, (_clsid_name, _iid_name, clsid, iid)) in attempts.iter().enumerate() {
                let hr = CoCreateInstance(
                    clsid,
                    ptr::null_mut(),
                    CLSCTX_ALL.0,
                    iid,
                    &mut policy_config,
                );

                if hr.is_ok() && !policy_config.is_null() {
                    success = true;
                    use_vista_interface = i >= 2;
                    break;
                }
                last_error = Some(hr);
                policy_config = ptr::null_mut();
            }

            if !success || policy_config.is_null() {
                CoUninitialize();
                return Err(format!(
                    "無法創建 IPolicyConfig 實例 (嘗試了所有 CLSID/IID 組合): {:?}",
                    last_error
                ));
            }

            let device_id_wide: Vec<u16> =
                device_id.encode_utf16().chain(std::iter::once(0)).collect();
            let device_id_pcwstr = PCWSTR::from_raw(device_id_wide.as_ptr());

            let hr1: HRESULT;
            let hr2: HRESULT;

            if use_vista_interface {
                let policy = policy_config as *mut IPolicyConfigVista;
                let vtable = (*policy).vtable;
                let set_default_fn = (*vtable).set_default_endpoint;

                hr1 = set_default_fn(policy_config, device_id_pcwstr, eConsole);
                hr2 = set_default_fn(policy_config, device_id_pcwstr, eCommunications);

                let release_fn = (*vtable).release;
                release_fn(policy_config);
            } else {
                let policy = policy_config as *mut IPolicyConfig;
                let vtable = (*policy).vtable;
                let set_default_fn = (*vtable).set_default_endpoint;

                hr1 = set_default_fn(policy_config, device_id_pcwstr, eConsole);
                hr2 = set_default_fn(policy_config, device_id_pcwstr, eCommunications);

                let release_fn = (*vtable).release;
                release_fn(policy_config);
            }

            CoUninitialize();

            if hr1.is_err() && hr2.is_err() {
                return Err(format!(
                    "設定默認設備失敗: eConsole={:?}, eCommunications={:?}",
                    hr1, hr2
                ));
            }

            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 獲取系統預設音訊輸出裝置的音量
/// 返回音量值 (0.0 ~ 1.0)
#[tauri::command]
pub fn get_default_device_volume() -> Result<f32, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("無法取得默認設備: {:?}", e))?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| format!("無法連接音量控制接口: {:?}", e))?;

            let volume = endpoint
                .GetMasterVolumeLevelScalar()
                .map_err(|e| format!("無法取得音量: {:?}", e))?;

            CoUninitialize();
            Ok(volume)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 設定系統預設音訊輸出裝置的音量
/// @param volume 音量大小 (0.0 ~ 1.0)
#[tauri::command]
pub fn set_default_device_volume(volume: f32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        let volume = volume.max(0.0).min(1.0);

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("無法取得默認設備: {:?}", e))?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| format!("無法連接音量控制接口: {:?}", e))?;

            endpoint
                .SetMasterVolumeLevelScalar(volume, std::ptr::null())
                .map_err(|e| format!("無法設定音量: {:?}", e))?;

            CoUninitialize();
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 獲取系統預設音訊輸出裝置的靜音狀態
/// 返回是否靜音 (true=靜音, false=未靜音)
#[tauri::command]
pub fn get_default_device_mute() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("無法取得默認設備: {:?}", e))?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| format!("無法連接音量控制接口: {:?}", e))?;

            let is_muted = endpoint
                .GetMute()
                .map_err(|e| format!("無法取得靜音狀態: {:?}", e))?;

            CoUninitialize();
            Ok(is_muted.as_bool())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}

/// 設定系統預設音訊輸出裝置的靜音狀態
/// @param mute 是否靜音 (true=靜音, false=取消靜音)
#[tauri::command]
pub fn set_default_device_mute(mute: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("無法獲取設備資料清單: {:?}", e))?;

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("無法取得默認設備: {:?}", e))?;

            let endpoint: IAudioEndpointVolume = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| format!("無法連接音量控制接口: {:?}", e))?;

            endpoint
                .SetMute(BOOL::from(mute), std::ptr::null())
                .map_err(|e| format!("無法設定靜音狀態: {:?}", e))?;

            CoUninitialize();
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("音樂控制 只支持 Windows :(((".to_string())
    }
}
