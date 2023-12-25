import React, { useState } from 'react'
import { Divider, Form, Input, Select, Space, Switch } from 'antd'

interface ApiSettingProps {
  code: ManagerCode
  token?: string
  config?: ApiConfig
}

const ApiSetting = ({ code, token: defaultToken, config }: ApiSettingProps) => {
  console.log(config)

  const [token, setToken] = useState(defaultToken)

  const disabled = code === 'SMMS'

  return (
    <>
      <Form.Item label="TOKEN">
        <Input.Password
          placeholder="输入 token"
          value={token || ''}
          onChange={(e) => setToken(e.target.value)}
        />
      </Form.Item>

      <Divider>图片列表</Divider>

      <Form.Item label="接口">
        <Input
          placeholder="输入图片列表接口"
          defaultValue={config?.list.url}
          disabled={disabled}
        />
      </Form.Item>

      <Form.Item label="请求方法">
        <Select defaultValue={config?.list.method.type} disabled={disabled}>
          <Select.Option value="GET">GET</Select.Option>
          <Select.Option value="POST">POST</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item>
        <Space wrap>
          <Form.Item label="图片数组键">
            <Input
              placeholder="输入图片数组键名"
              defaultValue={config?.list.controller.items_key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="图片地址键">
            <Input
              placeholder="输入图片地址键名"
              defaultValue={config?.list.controller.image_url_key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="图片删除 id 键">
            <Input
              placeholder="输入图片删除 id 键名"
              defaultValue={config?.list.controller.deleted_id_key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="图片缓存键">
            <Input
              placeholder="输入图片缓存键名"
              defaultValue={config?.list.controller.thumb_key ?? ''}
              disabled={disabled}
            />
          </Form.Item>
        </Space>
      </Form.Item>

      <Divider>删除</Divider>

      <Form.Item label="接口">
        <Input
          placeholder="输入图片删除接口"
          defaultValue={config?.delete.url}
          disabled={disabled}
        />
      </Form.Item>

      <Form.Item label="请求方法">
        <Select defaultValue={config?.delete.method.type} disabled={disabled}>
          <Select.Option value="GET">GET</Select.Option>
          <Select.Option value="POST">POST</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item
        label="是否有响应体"
        tooltip="删除图片如果有响应体则应该传入响应体中的有效属性键，否则以响应状态码判断是否成功"
      >
        <Switch
          defaultValue={config?.delete.controller.type === 'JSON'}
          disabled={disabled}
        />
      </Form.Item>

      {config?.delete.controller.type === 'JSON' ? (
        <Form.Item>
          <Form.Item label="成功键">
            <Input
              placeholder="删除成功与否的键名"
              defaultValue={config?.delete.controller.key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="成功的值" tooltip="删除成功时值应该是什么">
            <Input
              addonBefore={
                <Select
                  defaultValue={typeof config?.delete.controller.should_be}
                  disabled={disabled}
                >
                  <Select.Option value="boolean">布尔</Select.Option>
                  <Select.Option value="number">数字</Select.Option>
                  <Select.Option value="string">字符串</Select.Option>
                </Select>
              }
              placeholder="删除成功的值"
              defaultValue={config?.delete.controller.should_be}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="失败消息键">
            <Input
              placeholder="删除失败的消息键名"
              defaultValue={config?.delete.controller.message_key}
              disabled={disabled}
            />
          </Form.Item>
        </Form.Item>
      ) : null}

      <Divider>上传</Divider>
    </>
  )
}

export default ApiSetting
