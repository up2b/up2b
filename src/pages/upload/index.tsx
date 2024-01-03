import React, { useEffect, useState } from 'react'
import { appWindow } from '@tauri-apps/api/window'
import { TauriEvent, UnlistenFn } from '@tauri-apps/api/event'
import { message, Progress, Flex, Spin } from 'antd'
import { CloudUploadOutlined, PlusOutlined } from '@ant-design/icons'
import { filterImages } from '~/lib/image'
import {
  addImagesInStorage,
  getConfig,
  uploadImage,
  getCompressState,
  getSupportStream,
  getAllowedFormats,
} from '~/lib'
import { suspense } from '~/advance'
import { LazyUploadResult } from '~/lazy'
import './index.scss'

interface Image {
  path: string
  status: UploadStatus | number
  compressing: boolean
}

const formatBytes = (bytes: number): string => {
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']

  if (bytes === 0) return '0 Byte'

  const i = parseInt(
    Math.floor(Math.log(bytes) / Math.log(1024)).toString(),
    10,
  )
  const formattedSize = parseFloat((bytes / Math.pow(1024, i)).toFixed(2))

  return `${formattedSize} ${sizes[i]}`
}

const Upload = () => {
  const [messageApi, contextHolder] = message.useMessage()

  const [config, setConfig] = useState<Config | null>(null)

  const [images, setImages] = useState<Image[]>([])

  const [allowedFormats, setAllowedFormats] = useState<AllowedImageFormat[]>([])
  const [compressState, setCompressState] = useState(false)
  const [supportStream, setSupportStream] = useState(true)

  useEffect(() => {
    if (config) return

    getConfig().then((c) => {
      if (!c)
        messageApi.warning('配置文件不存在，请先在设置页面选择并配置一个图床')
      else {
        setConfig(c)

        getCompressState().then((b) => setCompressState(b))
        getAllowedFormats().then((fs) => setAllowedFormats(fs))
        getSupportStream().then((b) => {
          setSupportStream(b)

          if (!b) messageApi.warning('当前图床不支持流式上传，无法显示上传进度')
        })
      }
    })
  }, [])

  useEffect(() => {
    if (!config || !allowedFormats) return

    const unlisten = appWindow.listen<string[]>(
      TauriEvent.WINDOW_FILE_DROP,
      async (e) => {
        const imgs = filterImages(e.payload, allowedFormats).map((item) => ({
          path: item,
          status: 0,
          compressing: false,
        }))

        !imgs.length &&
          messageApi.warning('选择的所有文件格式都不在允许上传的范围内')

        setImages(imgs)

        for (const [index, image] of imgs.entries()) {
          await uploadOne(index, image, config.using)
        }
      },
    )

    return () => {
      unlisten.then((f) => f())
    }
  }, [config, allowedFormats])

  const uploadOne = async (
    index: number,
    image: Image,
    imageBed: ManagerCode,
  ) => {
    const config = await getConfig()
    if (!config) {
      setImages([])
      messageApi.error('配置为空，请先选择并配置图床')

      return
    }

    let uploadListener: UnlistenFn | null = null
    if (supportStream) {
      uploadListener = await appWindow.listen<Progress>(
        'upload://progress',
        (e) => {
          setImages((pre) => {
            const arr = [...pre]
            arr[index].status =
              Math.round((e.payload.progress / e.payload.total) * 1000) / 10

            return arr
          })
        },
      )
    }

    let automaticCompression = compressState && config.automatic_compression

    let compressListener: UnlistenFn | null = null
    if (automaticCompression) {
      compressListener = await appWindow.listen<CompressEvent>(
        'upload://compress',
        (e) => {
          if (e.payload.type === 'END') {
            const { filename, original, compressed } = e.payload
            messageApi.success(
              `${filename} 已压缩：${formatBytes(original)} -> ${formatBytes(
                compressed,
              )}`,
            )
          }

          setImages((pre) => {
            const arr = [...pre]
            arr[index].compressing = e.payload.type === 'START'

            return arr
          })
        },
      )
    }

    const resp = await uploadImage(image.path)

    if (uploadListener) uploadListener()

    if (compressListener) compressListener()

    if (resp.type === 'Error') {
      if (resp.code === 'REPEATED') {
        const url = 'https' + resp.detail.split('https')[1]
        messageApi.warning('图片重复上传：' + url)

        setImages((pre) => {
          const arr = [...pre]
          arr[index].status = {
            type: 'warning',
            src: url,
            msg: '重复上传',
          }

          return arr
        })
      } else {
        messageApi.error(resp.detail)

        setImages((pre) => {
          const arr = [...pre]
          arr[index].status = {
            type: 'error',
            error: resp.code,
          }

          return arr
        })
      }

      return
    }

    setImages((pre) => {
      const arr = [...pre]
      arr[index].status = {
        type: 'success',
        src: resp.url,
        thumb: resp.thumb,
        deleteId: resp.deleted_id,
        deleteImage,
      }

      return arr
    })

    addImagesInStorage(imageBed, {
      url: resp.url,
      deleted_id: resp.deleted_id,
    })
  }

  const deleteImage = (url: string) => {
    setImages((pre) =>
      pre.filter(
        (pre) =>
          typeof pre.status === 'number' ||
          !(pre.status.type === 'success' && pre.status.src === url),
      ),
    )
  }

  return (
    <>
      {contextHolder}

      <div className="upload">
        {images.length === 0 ? (
          <>
            <CloudUploadOutlined className="upload-icon" />
            <p className="upload-text">拖拽图片到本窗口或点击选择图片</p>
            <p className="upload-allowed-formats">
              ({allowedFormats.join('、')})
            </p>
          </>
        ) : (
          <Flex wrap="wrap" gap="small" justify="center" align="center">
            {images.map((image, index) => (
              <Spin tip="压缩中..." spinning={image.compressing} key={index}>
                <div className="upload-status">
                  {typeof image.status === 'object' ? (
                    suspense(
                      <LazyUploadResult
                        imageBedCode={config!.using}
                        messageApi={messageApi}
                        status={image.status as UploadStatus}
                      />,
                    )
                  ) : (
                    <div className="upload-status-progress">
                      {image.compressing ? null : supportStream ? (
                        <Progress
                          type="circle"
                          percent={image.status as number}
                        />
                      ) : (
                        <Spin />
                      )}
                    </div>
                  )}
                </div>
              </Spin>
            ))}

            {images.length &&
            typeof images[images.length - 1].status === 'object' ? (
              <div className="upload-add">
                <PlusOutlined />
              </div>
            ) : null}
          </Flex>
        )}
      </div>
    </>
  )
}

export default Upload
