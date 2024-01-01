import { invoke } from '@tauri-apps/api'

export const getConfig = async () => {
  const config = await invoke<Config | null>('get_config')
  return config
}

export const updateConfig = async (config: Config) => {
  await invoke('update_config', { config })
}

export const uploadImage = async (imagePath: string) => {
  return await invoke<UploadResult>('upload_image', { imagePath })
}

export const getAllImages = async () => {
  return await invoke<ImageResponseItem[]>('get_all_images')
}

export const deleteImage = async (deleteId: string) => {
  return await invoke<DeleteResponse>('delete_image', { deleteId })
}

export const getCompressState = async () => {
  return await invoke<boolean>('compress_state')
}

export const getSupportStream = async () => {
  return await invoke<boolean>('support_stream')
}

export const verify = async <T extends APIManagerKey | CheveretoManagerKey>(
  imageBed: T,
  config: InferAuthConfigKind<T>,
) => {
  return await invoke<Extra | null>('verify', {
    imageBed,
    config,
  })
}

export const getUsingImageBed = async () => {
  return await invoke<ManagerCode>('get_using_image_bed')
}

export const getAllowedFormats = async () => {
  return await invoke<AllowedImageFormat[]>('allowed_formats')
}

export const getSmmsConfig = async () => {
  return await invoke<ApiConfig>('smms_config')
}

export const getImageBeds = async () => {
  return await invoke<ManagerItem[]>('get_managers')
}

export const newCustomManager = async (
  managerCode: string,
  authConfig: ApiAuthConfig,
) => {
  return await invoke('new_custom_manager', { managerCode, authConfig })
}
