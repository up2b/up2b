import React, { useEffect, useState } from 'react'
import { Form, Button, Space } from 'antd'
import { areObjectsEqual } from '~/lib'
import { getSmmsConfig } from '~/lib/api'
import ApiSetting, {
  initApiConfigFormValues,
  apiConfigToForm,
  formDataToApiConfig,
} from '.'

interface ApiSettingProps {
  code: string
  authConfig?: ApiAuthConfig
  onOk?: (authConfig: ApiAuthConfig) => Promise<void>
  disableCancelButton?: boolean
  disableOkButton?: boolean
}

const ApiSettingForm = ({
  code,
  authConfig,
  onOk,
  disableCancelButton,
  disableOkButton,
}: ApiSettingProps) => {
  const [form] = Form.useForm()

  const [config, setConfig] = useState<ApiConfigForm | undefined>(
    apiConfigToForm(authConfig?.api),
  )

  const [innerDisableOkButton, setInnerDisableOkButton] = useState(true)

  useEffect(() => {
    if (config) {
      form.resetFields()
    }
  }, [code, config])

  useEffect(() => {
    if (!authConfig?.api) {
      if (code === 'SMMS') {
        // 第一次配置 smms 时才会获取 smms 示例配置
        getSmmsConfig().then((c) => {
          setConfig(apiConfigToForm(c))
        })
      } else {
        setConfig(initApiConfigFormValues.api)
      }
    } else {
      setConfig(apiConfigToForm(authConfig.api))
    }
  }, [code, authConfig])

  if (!config) {
    return null
  }

  const formData: ApiAuthConfigForm = {
    type: 'API',
    token: authConfig?.token ?? '',
    api: config,
  }

  return (
    <Form
      form={form}
      initialValues={formData}
      onValuesChange={(_, allValues) => {
        setInnerDisableOkButton(
          areObjectsEqual(formData, { type: 'API', ...allValues }),
        )
      }}
      onFinish={async (values) => {
        const c = formDataToApiConfig(values.api)

        await onOk?.({ ...values, type: 'API', api: c })

        setInnerDisableOkButton(true)
      }}
    >
      <ApiSetting code={code} />

      <Form.Item
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          marginBottom: '2rem',
        }}
      >
        <Space>
          <Button
            key="cancel"
            type="default"
            onClick={() => location.reload()}
            disabled={disableCancelButton}
          >
            取消
          </Button>
          <Button
            key="submit"
            type="primary"
            htmlType="submit"
            disabled={disableOkButton && innerDisableOkButton}
          >
            保存
          </Button>
        </Space>
      </Form.Item>
    </Form>
  )
}

export default ApiSettingForm
