import React from 'antd'
import { Form, Input, Radio, Space } from 'antd'
import type { FormRule } from 'antd'

interface ImageListProps {
  data: ApiAuthConfig
  rules: FormRule[]
  urlRules: FormRule[]
  disabled: boolean
  handleChange: (data: ApiAuthConfig) => void
}

const ImageList = ({
  data,
  rules,
  urlRules,
  disabled,
  handleChange,
}: ImageListProps) => {
  const name = (...key: string[]) => ['api', 'list', ...key]

  return (
    <>
      <Form.Item name={name('url')} label="接口" rules={urlRules}>
        <Input
          placeholder="输入图片列表接口"
          value={data.api.list.url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: { ...data.api.list, url: e.target.value },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item name={name('method', 'type')} label="请求方法" rules={rules}>
        <Radio.Group
          value={data.api.list.method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: {
                  ...data.api.list,
                  method: { ...data.api.list.method, type: e.target.value },
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
            value={data.api.list.controller.items_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...data.api.list.controller,
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
            value={data.api.list.controller.image_url_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...data.api.list.controller,
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
            value={data.api.list.controller.deleted_id_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...data.api.list.controller,
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
            value={data.api.list.controller.thumb_key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  list: {
                    ...data.api.list,
                    controller: {
                      ...data.api.list.controller,
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
