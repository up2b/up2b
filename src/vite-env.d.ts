/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly VITE_APP_TITLE: string
    readonly TAURI_PLATFORM: 'windows' | 'macos'
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}