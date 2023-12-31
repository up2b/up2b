import React from 'antd'
import { Form, Input, Radio, Space } from 'antd'
import type { FormRule } from 'antd'

interface ImageListProps {
  rules: FormRule[]
  pathRules: FormRule[]
  disabled: boolean
}

const ImageList = ({ rules, pathRules, disabled }: ImageListProps) => {
  const name = (...key: string[]) => ['api', 'list', ...key]

  return (
    <>
      <Form.Item name={name('path')} label="路径" rules={pathRules}>
        <Input placeholder="输入图片列表接口路径" disabled={disabled} />
      </Form.Item>

      <Form.Item name={name('method', 'type')} label="请求方法" rules={rules}>
        <Radio.Group disabled={disabled}>
          <Radio value="GET">GET</Radio>
          <Radio value="POST">POST</Radio>
        </Radio.Group>
      </Form.Item>

      <Space wrap>
        <Form.Item
          label="图片数组键"
          name={name('controller', 'items_key')}
          rules={rules}
        >
          <Input placeholder="输入图片数组键名" disabled={disabled} />
        </Form.Item>

        <Form.Item
          label="图片地址键"
          name={name('controller', 'image_url_key')}
          rules={rules}
        >
          <Input placeholder="输入图片地址键名" disabled={disabled} />
        </Form.Item>

        <Form.Item
          label="图片删除 id 键"
          name={name('controller', 'deleted_id_key')}
          rules={rules}
        >
          <Input placeholder="输入图片删除 id 键名" disabled={disabled} />
        </Form.Item>

        <Form.Item name={name('controller', 'thumb_key')} label="图片缓存键">
          <Input placeholder="输入图片缓存键名" disabled={disabled} />
        </Form.Item>
      </Space>
    </>
  )
}

export default ImageList
