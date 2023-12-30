import React from 'antd'
import { Form, Input, Radio, Space } from 'antd'
import type { FormRule } from 'antd'

interface ImageListProps {
  data: ApiAuthConfig
  rules: FormRule[]
  pathRules: FormRule[]
  disabled: boolean
  handleChange: (data: ApiAuthConfig) => void
}

const ImageList = ({
  data,
  rules,
  pathRules,
  disabled,
  handleChange,
}: ImageListProps) => {
  const name = (...key: string[]) => ['api', 'list', ...key]

  const { path, method, controller } = data.api.list

  return (
    <>
      <Form.Item name={name('path')} label="路径" rules={pathRules}>
        <Input
          placeholder="输入图片列表接口路径"
          value={path}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: { ...data.api.list, path: e.target.value },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item name={name('method', 'type')} label="请求方法" rules={rules}>
        <Radio.Group
          value={method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: {
                  ...data.api.list,
                  method: { ...method, type: e.target.value },
                },
              },
            })
          }
        >
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
          <Input
            placeholder="输入图片数组键名"
            value={controller.items_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...controller,
                      items_key: e.target.value,
                    },
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item
          label="图片地址键"
          name={name('controller', 'image_url_key')}
          rules={rules}
        >
          <Input
            placeholder="输入图片地址键名"
            value={controller.image_url_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...controller,
                      image_url_key: e.target.value,
                    },
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item
          label="图片删除 id 键"
          name={name('controller', 'deleted_id_key')}
          rules={rules}
        >
          <Input
            placeholder="输入图片删除 id 键名"
            value={controller.deleted_id_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...controller,
                      deleted_id_key: e.target.value,
                    },
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item name={name('controller', 'thumb_key')} label="图片缓存键">
          <Input
            placeholder="输入图片缓存键名"
            value={controller.thumb_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...controller,
                      thumb_key: e.target.value,
                    },
                  },
                },
              })
            }
          />
        </Form.Item>
      </Space>
    </>
  )
}

export default ImageList
