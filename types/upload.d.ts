interface SuccessStatus {
  type: 'success'
  src: string
  thumb?: string
  deleteId: string
  deleteImage: (url: string) => void
}

interface WarningStatus {
  type: 'warning'
  src: string
  msg: string
}

interface ErrorStatus {
  type: 'error'
  error: string
}

type UploadStatus = SuccessStatus | WarningStatus | ErrorStatus

type AllowedImageFormat = 'PNG' | 'JPEG' | 'GIF' | 'WEBP' | 'BMP' | 'AVIF'

interface NoCompressEvent {
  type: 'NO'
}

interface StartCompressEvent {
  type: 'START'
}

interface EndCompressEvent {
  type: 'END'
  filename: string
  original: number
  compressed: number
}

type CompressEvent = NoCompressEvent | StartCompressEvent | EndCompressEvent
