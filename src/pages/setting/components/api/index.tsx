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
  authConfig?: ApiAuthConfig
  onChange?: (data: ApiAuthConfig) => void
}

export const initApiConfig: ApiAuthConfig = {
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
      max_size: 0,
      timeout: 0,
      allowed_formats: ['PNG', 'JPEG', 'GIF'],
      content_type: {
        type: 'MULTIPART',
        file_kind: 'STREAM',
        file_part_name: '',
      },
      other_body: {},
      controller: {
        image_url_key: '',
        deleted_id_key: '',
      },
    },
  },
}

const ApiSetting = ({ code, authConfig, onChange }: ApiSettingProps) => {
  const [token, setToken] = useState<string>(authConfig?.token ?? '')
  const [config, setConfig] = useState<ApiConfig | undefined>(authConfig?.api)

  useEffect(() => {
    if (!authConfig?.api) {
      if (code === 'SMMS') {
        // 第一次配置 smms 时才会获取 smms 示例配置
        getSmmsConfig().then((c) => {
          setConfig(c)
        })
      } else {
        setConfig(initApiConfig.api)
      }
    }
  }, [code, config])

  const disabled = code === 'SMMS'

  const handleChange = (data: ApiAuthConfig) => {
    setToken(data.token)
    setConfig(data.api)

    onChange?.(data)
  }

  const rules: FormRule[] = [{ required: true }]
  const pathRules: FormRule[] = [
    ...rules,
    { type: 'string', warningOnly: true, pattern: /^\/\w+\/?$/ },
  ]

  if (!config) {
    return null
  }

  const data: ApiAuthConfig = {
    type: 'API',
    token,
    api: config,
  }

  return (
    <>
      <Form.Item name="token" label="TOKEN" rules={rules}>
        <Input.Password
          placeholder="输入 token"
          value={token ?? ''}
          onChange={(e) => handleChange({ ...data, token: e.target.value })}
        />
      </Form.Item>

      <Form.Item
        name={['api', 'base_url']}
        label="接口"
        rules={[...rules, { type: 'url', warningOnly: true }]}
      >
        <Input
          placeholder="输入接口"
          disabled={disabled}
          // value={data.api.base_url}
          onChange={(e) =>
            handleChange({
              ...data,
              api: { ...data.api, base_url: e.target.value },
            })
          }
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
        pathRules={pathRules}
        disabled={disabled}
        handleChange={handleChange}
      />

      <Divider>删除</Divider>
      <Delete
        data={data}
        rules={rules}
        pathRules={pathRules}
        disabled={disabled}
        handleChange={handleChange}
      />

      <Divider>上传</Divider>
      <Upload
        data={data}
        rules={rules}
        pathRules={pathRules}
        disabled={disabled}
        handleChange={handleChange}
      />
    </>
  )
}

export default ApiSetting
