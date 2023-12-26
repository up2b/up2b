import React, { useState } from 'react'
import {
  Divider,
  Form,
  Input,
  InputNumber,
  Radio,
  Select,
  Space,
  Switch,
} from 'antd'

interface ApiSettingProps {
  code: ManagerCode
  token?: string
  config?: ApiConfig
}

const ALLOWED_FORMATS = ['PNG', 'JPEG', 'GIF', 'WEBP', 'BMP']

const ApiSetting = ({ code, token: defaultToken, config }: ApiSettingProps) => {
  console.log(config)

  const [token, setToken] = useState(defaultToken)

  const [selectedFormats, setSelectedFormats] = useState<string[]>(
    config?.upload.allowed_formats ?? [],
  )

  const filteredFormats = ALLOWED_FORMATS.filter(
    (o) => !selectedFormats.includes(o),
  )

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

      <Form.Item>
        <Form.Item label="接口">
          <Input
            placeholder="输入上传图片接口"
            defaultValue={config?.upload.url}
            disabled={disabled}
          />
        </Form.Item>

        <Form.Item label="最大体积">
          <InputNumber
            placeholder="输入允许的最大体积"
            defaultValue={config?.upload.max_size / 1024 / 1024}
            disabled={disabled}
            addonAfter="MB"
          />
        </Form.Item>

        <Form.Item label="允许的格式">
          <Select
            mode="multiple"
            placeholder="选择图片格式"
            value={selectedFormats}
            onChange={setSelectedFormats}
            options={filteredFormats.map((item) => ({
              value: item,
              label: item,
            }))}
            disabled={disabled}
          />
        </Form.Item>

        <Form.Item>
          <Form.Item label="请求体类型">
            <Select
              defaultValue={config?.upload.content_type.type}
              disabled={disabled}
              options={[
                {
                  value: 'JSON',
                  label: 'json',
                },
                {
                  value: 'MULTIPART',
                  label: 'multipart',
                },
              ]}
            />
          </Form.Item>

          {config?.upload.content_type.type === 'MULTIPART' ? (
            <>
              <Form.Item
                label="上传类型"
                tooltip="流式响应支持上传进度，流式上传失败时可尝试更换为 bytes"
              >
                <Radio.Group
                  defaultValue={
                    config?.upload.content_type.file_kind ?? 'STREAM'
                  }
                  disabled={disabled}
                >
                  <Radio value="STREAM">流</Radio>
                  <Radio value="BUFFER">bytes</Radio>
                </Radio.Group>
              </Form.Item>

              <Form.Item label="图片的表单键">
                <Input
                  defaultValue={config.upload.content_type.file_part_name}
                  disabled={disabled}
                />
              </Form.Item>
            </>
          ) : (
            <>
              {/*TODO: 以后再完善 json 上传*/}
              <Form.Item label="图片数组的表单键">
                <Input defaultValue={config?.upload.content_type.key} />
              </Form.Item>

              <Form.Item label="除图片之外的其他 json 数据">
                <Input.TextArea />
              </Form.Item>
            </>
          )}
        </Form.Item>

        <Form.Item>
          <Form.Item label="图片键">
            <Input
              defaultValue={config?.upload.controller.image_url_key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="删除 id 键">
            <Input
              defaultValue={config?.upload.controller.deleted_id_key}
              disabled={disabled}
            />
          </Form.Item>

          <Form.Item label="图片缓存键">
            <Input
              defaultValue={config?.upload.controller.thumb_key}
              disabled={disabled}
            />
          </Form.Item>
        </Form.Item>
      </Form.Item>
    </>
  )
}

export default ApiSetting
