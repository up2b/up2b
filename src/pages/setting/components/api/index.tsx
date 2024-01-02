import React from 'react'
import { Divider, Form, Input } from 'antd'
import type { FormRule } from 'antd'
import AuthMethod from './auth-method'
import ImageList from './image-list'
import Delete from './delete'
import Upload from './upload'

interface ApiSettingProps {
  code: string
}

export const initApiConfigFormValues: ApiAuthConfigForm = {
  type: 'API',
  token: '',
  api: {
    base_url: '',
    auth_method: {
      type: 'HEADER',
    },
    list: {
      path: '',
      method: { type: 'GET' },
      controller: {
        items_key: '',
        image_url_key: '',
        deleted_id_key: '',
      },
    },
    delete: {
      path: '',
      method: { type: 'GET', kind: { type: 'PATH' } },
      controller: { type: 'JSON', key: '', should_be: true },
    },
    upload: {
      path: '',
      max_size: undefined,
      timeout: 5,
      allowed_formats: ['PNG', 'JPEG', 'GIF'],
      content_type: {
        type: 'MULTIPART',
        file_kind: 'STREAM',
        file_part_name: '',
      },
      other_body: undefined,
      controller: {
        status: { key: 'status', value: true },
        error: { key: 'error' },
        success: {
          image_url_key: '',
          deleted_id_key: '',
        },
      },
    },
  },
}

export const apiConfigToForm = (
  config?: ApiConfig,
): ApiConfigForm | undefined => {
  if (!config) return undefined

  return {
    ...config,
    upload: {
      ...config.upload,
      other_body: config.upload.other_body
        ? JSON.stringify(config.upload.other_body)
        : undefined,
    },
  }
}

export const formDataToApiConfig = (formData: ApiConfigForm): ApiConfig => {
  const other_body = formData.upload.other_body
    ? JSON.parse(formData.upload.other_body)
    : undefined
  return {
    ...formData,
    upload: {
      ...formData.upload,
      max_size: formData.upload.max_size!,
      other_body,
    },
  }
}

const ApiSetting = ({ code }: ApiSettingProps) => {
  const rules: FormRule[] = [{ required: true }]
  const pathRules: FormRule[] = [
    ...rules,
    { type: 'string', warningOnly: true, pattern: /^\/\w+\/?$/ },
  ]

  const disabled = code === 'SMMS'

  return (
    <>
      <Form.Item name="token" label="TOKEN" rules={rules}>
        <Input.Password placeholder="输入 token" />
      </Form.Item>

      <Form.Item
        name={['api', 'base_url']}
        label="接口"
        rules={[...rules, { type: 'url', warningOnly: true }]}
      >
        <Input placeholder="输入接口" disabled={disabled} />
      </Form.Item>

      <Divider>认证方式</Divider>
      <AuthMethod rules={rules} disabled={disabled} />

      <Divider>图片列表</Divider>
      <ImageList rules={rules} pathRules={pathRules} disabled={disabled} />

      <Divider>删除</Divider>
      <Delete rules={rules} pathRules={pathRules} disabled={disabled} />

      <Divider>上传</Divider>
      <Upload rules={rules} pathRules={pathRules} disabled={disabled} />
    </>
  )
}

export default ApiSetting
