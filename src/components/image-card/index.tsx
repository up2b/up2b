import React, { useState } from 'react'
import { Card, Image, Spin, Tooltip } from 'antd'
import {
  EyeOutlined,
  CopyOutlined,
  DeleteOutlined,
  WarningOutlined,
} from '@ant-design/icons'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { MessageInstance } from 'antd/es/message/interface'
import { deleteImage } from '~/lib'
import './index.scss'

interface ImageCardStatusSuccess {
  type: 'success'
  deleteId: string
  afterDeleting: (url: string) => void
}

interface ImageCardStatusWarning {
  type: 'warning'
  msg: string
}

interface ImageCardProps {
  url: string
  thumb?: string
  messageApi: MessageInstance
  status: ImageCardStatusSuccess | ImageCardStatusWarning
}

const ImageCard = ({ url, thumb, messageApi, status }: ImageCardProps) => {
  const [visible, setVisible] = useState(false)
  const [deleting, setDeleting] = useState(false)

  const onDelete = async () => {
    if (status.type === 'warning') return

    const { deleteId, afterDeleting } = status

    setDeleting(true)

    const resp = await deleteImage(deleteId)

    setDeleting(false)

    if (resp.success) {
      messageApi.success('已删除')
      afterDeleting(url)
    } else {
      if (resp.error === '图片不存在') {
        messageApi.warning(resp.error)
        afterDeleting(url)
      } else {
        messageApi.error(resp.error ?? '未知错误')
      }
    }
  }

  return (
    <Card className="image-card" hoverable>
      <Spin tip="正在删除..." spinning={deleting}>
        <Image
          className="image-list-item"
          src={thumb ?? url}
          width={150}
          height={180}
          preview={{
            visible,
            src: url,
            mask: (
              <span className="image-list-actions">
                <EyeOutlined onClick={() => setVisible(true)} />
                <CopyOutlined
                  onClick={async () => {
                    await writeText(url)
                    messageApi.success('已复制到剪贴板')
                  }}
                />
                {status.type === 'success' ? (
                  <DeleteOutlined onClick={onDelete} />
                ) : (
                  <Tooltip
                    placement="topRight"
                    title={status.msg}
                    color="orange"
                  >
                    <WarningOutlined />
                  </Tooltip>
                )}
              </span>
            ),
            onVisibleChange: (value) => {
              if (!value) {
                setVisible(false)
              }
            },
          }}
        />
      </Spin>
    </Card>
  )
}

export default ImageCard
