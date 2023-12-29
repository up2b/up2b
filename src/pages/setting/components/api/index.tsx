import React, { useEffect, useState } from 'react'
import { Divider, Form, Input } from 'antd'
import type { FormRule } from 'antd'
import { getSmmsConfig } from '~/lib/api'
import AuthMethod from './auth-method'
import ImageList from './image-list'
import Delete from './delete'
import Upload from './upload'

interface ApiSettingProps {
  code: string
  token?: string
  config?: ApiConfig
  onChange?: (data: ApiAuthConfig) => void
}

export const initApiConfig: ApiAuthConfig = {
  type: 'API',
  token: '',
  api: {
    auth_method: {
      type: 'HEADER',
    },
    list: {
      url: '',
      method: { type: 'GET' },
      controller: {
        items_key: '',
        image_url_key: '',
        deleted_id_key: '',
      },
    },
    delete: {
      url: '',
      method: { type: 'GET', kind: { type: 'PATH' } },
      controller: { type: 'JSON', key: '', should_be: '' },
    },
    upload: {
      url: '',
      max_size: 0,
      timeout: 0,
      allowed_formats: ['PNG', 'JPEG', 'GIF'],
      content_type: {
        type: 'MULTIPART',
        file_kind: 'STREAM',
        file_part_name: '',
      },
      controller: {
        image_url_key: '',
        deleted_id_key: '',
      },
    },
  },
}

const ApiSetting = ({ code, token, config, onChange }: ApiSettingProps) => {
  const form = Form.useFormInstance()
  console.log(form.getFieldValue('upload'))

  const [data, setData] = useState<ApiAuthConfig>({
    type: 'API',
    token: token ?? '',
    api: config ?? initApiConfig.api,
  })

  useEffect(() => {
    if (code === 'SMMS' && !config) {
      getSmmsConfig().then((c) => {
        setData({ ...data, api: c })
      })
    }
  }, [code, config])

  const disabled = code === 'SMMS'

  const handleChange = (data: ApiAuthConfig) => {
    setData(data)

    onChange?.(data)
  }

  const rules: FormRule[] = [{ required: true }]
  const urlRules: FormRule[] = [...rules, { type: 'url', warningOnly: true }]

  console.log(data.api.upload.allowed_formats)

  return (
    <>
      <Form.Item name="token" label="TOKEN" rules={rules}>
        <Input.Password
          placeholder="输入 token"
          value={data.token || ''}
          onChange={(e) => handleChange({ ...data, token: e.target.value })}
        />
      </Form.Item>

      <Divider>认证方式</Divider>
      <AuthMethod
        data={data}
        rules={rules}
        disabled={disabled}
        handleChange={handleChange}
      />

      <Divider>图片列表</Divider>
      <ImageList
        data={data}
        rules={rules}
        urlRules={urlRules}
        disabled={disabled}
        handleChange={handleChange}
      />

      <Divider>删除</Divider>
      <Delete
        data={data}
        rules={rules}
        urlRules={urlRules}
        disabled={disabled}
        handleChange={handleChange}
      />

      <Divider>上传</Divider>
      <Upload
        data={data}
        rules={rules}
        urlRules={urlRules}
        disabled={disabled}
        handleChange={handleChange}
      />
    </>
  )
}

export default ApiSetting
