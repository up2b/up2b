interface BaseError {
  type: 'Error'
  detail: string
  code: string
}

interface DeleteResponse extends BaseError {
  success: boolean
  error: string | null
}

interface ImageResponseItem {
  deleted_id: string
  url: string
  thumb?: string
}

interface UploadResponse extends ImageResponseItem {
  type: 'Response'
}

type UploadResult = UploadResponse | BaseError
