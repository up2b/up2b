import React, { useEffect, useState } from 'react'
import { message, Flex, Spin, FloatButton } from 'antd'
import { SyncOutlined } from '@ant-design/icons'
import {
  setStorage,
  getConfig,
  getAllImages,
  getImagesInStorage,
  getUsingImageBed,
} from '~/lib'
import './index.scss'
import { LazyImageCard } from '~/lazy'
import { suspense } from '~/advance'

const ImageList = () => {
  const [messageApi, contextHolder] = message.useMessage()

  const [imageBedCode, setImageBedCode] = useState<ManagerCode | null>(null)

  const [images, setImages] = useState<ImageResponseItem[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (imageBedCode) return

    getUsingImageBed()
      .then((c) => setImageBedCode(c))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    if (!imageBedCode) return

    const cached = getImagesInStorage(imageBedCode)

    if (!cached?.length) updateImageList()
    else setImages(cached)
  }, [imageBedCode])

  useEffect(() => {
    if (!imageBedCode || images.length === 0) return

    setStorage(imageBedCode, images)
  }, [images, imageBedCode])

  const updateImageList = async () => {
    const config = await getConfig()
    if (!config) {
      messageApi.error('配置为空')
      return
    }

    setLoading(true)

    const list = await getAllImages()

    list.reverse()

    setLoading(false)

    setStorage(imageBedCode!, list)

    setImages(list)
  }

  const afterDeleting = (url: string) => {
    setImages((pre) => {
      const newImages = pre.filter((v) => v.url !== url)

      setStorage(imageBedCode!, newImages)

      return newImages
    })
  }

  return (
    <Spin spinning={loading}>
      <div id="image-list">
        {contextHolder}

        <Flex wrap="wrap" gap="small" justify="center">
          {images.map((item, index) => (
            <div key={index} className="image-card-container">
              {suspense(
                <LazyImageCard
                  url={item.url}
                  thumb={item.thumb}
                  status={{
                    type: 'success',
                    deleteId: item.deleted_id,
                    afterDeleting: afterDeleting,
                  }}
                  messageApi={messageApi}
                />,
              )}
            </div>
          ))}
        </Flex>

        <FloatButton
          icon={<SyncOutlined />}
          tooltip="刷新列表"
          onClick={updateImageList}
        />
      </div>
    </Spin>
  )
}

export default ImageList
