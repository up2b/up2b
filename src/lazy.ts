import { lazy } from 'react'

export const LazyUpload = lazy(() => import('~/pages/upload/index'))
export const LazyUploadResult = lazy(
  () => import('~/pages/upload/uploaded-result'),
)

export const LazyList = lazy(() => import('~/pages/list/index'))

export const LazySetting = lazy(() => import('~/pages/setting/index'))

export const LazyImageCard = lazy(() => import('~/components/image-card/index'))
