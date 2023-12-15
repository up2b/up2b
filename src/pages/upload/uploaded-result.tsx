import React from 'react'
import { CloseCircleOutlined } from '@ant-design/icons'
import { LazyImageCard } from '~/lazy'
import { suspense } from '~/advance'
import type { MessageInstance } from 'antd/es/message/interface'
import { deleteImagesInStorage } from '~/lib'

interface UploadedResultProps {
  imageBedCode: ManagerCode
  status: UploadStatus
  messageApi: MessageInstance
}

const UploadedResult = ({
  imageBedCode,
  status,
  messageApi,
}: UploadedResultProps) => {
  if (status.type === 'error') {
    return (
      <div className="upload-error">
        <div className="error-icon">
          <CloseCircleOutlined />
        </div>
        <span>{status.error}</span>
      </div>
    )
  }

  if (status.type === 'warning') {
    return suspense(
      <LazyImageCard
        url={status.src}
        messageApi={messageApi}
        status={status}
      />,
    )
  }

  const { src, deleteId, deleteImage, thumb } = status

  const afterDeleting = (url: string) => {
    deleteImage(url)
    deleteImagesInStorage(imageBedCode, {
      url,
      deleted_id: deleteId,
      thumb,
    })
  }

  return suspense(
    <LazyImageCard
      url={src}
      thumb={thumb}
      messageApi={messageApi}
      status={{
        type: 'success',
        deleteId,
        afterDeleting,
      }}
    />,
  )
}

export default UploadedResult
