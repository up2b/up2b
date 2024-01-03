import React from 'react'
import { Form, Input, Radio, Space, InputNumber, Select } from 'antd'
import type { FormRule } from 'antd'
import Status from '../status'

const ALLOWED_FORMATS = ['PNG', 'JPEG', 'GIF', 'WEBP', 'BMP']

interface UploadProps {
  rules: FormRule[]
  pathRules: FormRule[]
  disabled: boolean
}

const Upload = ({ rules, pathRules, disabled }: UploadProps) => {
  const name = (...key: string[]) => ['api', 'upload', ...key]

  const form = Form.useFormInstance()

  const allowedFormatsValue =
    Form.useWatch(name('allowed_formats'), form) || ([] as string[])
  const contentTypeValue = Form.useWatch(
    name('content_type', 'type'),
    form,
  ) as ApiUploadConfig['content_type']['type']

  const filteredFormats = ALLOWED_FORMATS.filter(
    (o) => !allowedFormatsValue.includes(o),
  )

  return (
    <>
      <Form.Item label="路径" name={name('path')} rules={pathRules}>
        <Input placeholder="上传图片接口路径，如 /upload" disabled={disabled} />
      </Form.Item>

      <Space wrap>
        <Form.Item
          label="最大体积"
          name={name('max_size')}
          rules={rules}
          style={{ maxWidth: 240 }}
        >
          <InputNumber
            placeholder="允许的最大体积"
            disabled={disabled}
            addonAfter="MB"
          />
        </Form.Item>

        <Form.Item
          label="超时时间"
          name={name('timeout')}
          style={{ maxWidth: 240 }}
        >
          <InputNumber disabled={disabled} min={0} addonAfter="秒" />
        </Form.Item>
      </Space>

      <Form.Item
        label="允许的格式"
        name={name('allowed_formats')}
        rules={rules}
      >
        <Select
          mode="multiple"
          allowClear
          placeholder="选择图片格式"
          options={filteredFormats.map((item) => ({
            value: item,
            label: item,
          }))}
          disabled={disabled}
        />
      </Form.Item>

      <Form.Item name={name('content_type', 'type')} label="请求体类型">
        <Radio.Group disabled={disabled}>
          <Radio value="MULTIPART">multipart</Radio>
          <Radio value="JSON">json</Radio>
        </Radio.Group>
      </Form.Item>

      {contentTypeValue === 'MULTIPART' ? (
        <Space wrap>
          <Form.Item
            name={name('content_type', 'file_kind')}
            label="上传类型"
            tooltip="流式响应支持上传进度，流式上传失败时可尝试更换为 bytes"
          >
            <Radio.Group disabled={disabled}>
              <Radio value="STREAM">流</Radio>
              <Radio value="BUFFER">bytes</Radio>
            </Radio.Group>
          </Form.Item>

          <Form.Item
            label="图片的表单键"
            name={name('content_type', 'file_part_name')}
            rules={rules}
          >
            <Input disabled={disabled} />
          </Form.Item>
        </Space>
      ) : (
        <>
          {/*TODO: 以后再完善 json 上传*/}
          <Form.Item
            label="图片数组的表单键"
            name={name('content_type', 'key')}
            rules={rules}
          >
            <Input />
          </Form.Item>
        </>
      )}

      <Form.Item
        label="其他表单值"
        name={name('other_body')}
        extra="json 格式字符串，中文的单双引号会自动转换为英文双引号"
        normalize={(value: string | undefined) => {
          if (!value) return undefined
          return value
            .replaceAll("'", '"')
            .replaceAll('‘', '"')
            .replaceAll('’', '"')
            .replaceAll('“', '"')
            .replaceAll('”', '"')
        }}
        rules={[
          () => ({
            validator(_, value) {
              if (!value) return Promise.resolve()
              try {
                JSON.parse(value)
                return Promise.resolve()
              } catch (e) {
                return Promise.reject('格式不正确')
              }
            },
          }),
        ]}
      >
        <Input.TextArea placeholder='{"key": "value"}' disabled={disabled} />
      </Form.Item>

      <Space wrap>
        <Form.Item
          label="成功状态键"
          name={name('controller', 'status', 'key')}
          rules={rules}
        >
          <Input disabled={disabled} />
        </Form.Item>

        <Form.Item
          label="成功状态值"
          name={name('controller', 'status', 'value')}
          rules={rules}
        >
          <Status disabled={disabled} />
        </Form.Item>
      </Space>

      <Space wrap>
        <Form.Item
          label="图片键"
          name={name('controller', 'success', 'image_url_key')}
          rules={rules}
        >
          <Input disabled={disabled} />
        </Form.Item>

        <Form.Item
          label="删除 id 键"
          name={name('controller', 'success', 'deleted_id_key')}
          rules={rules}
        >
          <Input disabled={disabled} />
        </Form.Item>

        <Form.Item
          name={name('controller', 'success', 'thumb_key')}
          label="缩略图键"
        >
          <Input disabled={disabled} />
        </Form.Item>
      </Space>

      <Form.Item
        name={name('controller', 'error', 'key')}
        label="失败消息键"
        rules={rules}
      >
        <Input disabled={disabled} />
      </Form.Item>

      <Form.Item
        name={name('controller', 'error', 'repeated_regex')}
        label="重复上传正则表达式"
      >
        <Input disabled={disabled} />
      </Form.Item>
    </>
  )
}

export default Upload
