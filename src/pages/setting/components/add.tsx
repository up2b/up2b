import React, { useEffect, useState } from 'react'
import { Button, Form, Input, Modal, Space, message } from 'antd'
import ApiSetting, { initApiConfigFormValues } from './api'
import { CheckOutlined } from '@ant-design/icons'
import { checkNewManagerCode, newCustomManager } from '~/lib'
import type { MessageInstance } from 'antd/es/message/interface'

interface ManagerCodeState {
  checked: boolean
  code: string
}

interface NewApiCodeFormProps {
  code: ManagerCodeState
  setCode: React.Dispatch<React.SetStateAction<ManagerCodeState>>
}

const NewApiCodeForm = ({ code, setCode }: NewApiCodeFormProps) => {
  const [form] = Form.useForm()

  return (
    <Form
      form={form}
      onFinish={(values) => {
        setCode({ ...values, checked: true })
      }}
    >
      <Space.Compact>
        <Form.Item
          name="code"
          label="图床代码"
          tooltip="仅支持字母、数字、下划线。通常为图床名称，如smms、imgtg"
          hasFeedback
          normalize={(v: string) => {
            return v.toUpperCase()
          }}
          rules={[
            { required: true },
            {
              type: 'string',
              pattern: CODE_REGEX,
              warningOnly: true,
              message: '仅支持字母、数字、下划线',
            },
            {
              validator: async (_, value) => {
                if (!(await checkNewManagerCode('CUSTOM-' + value))) {
                  throw new Error('图床代码已存在')
                } else {
                  return
                }
              },
            },
          ]}
        >
          <Input placeholder="图床代码" value={code.code} allowClear />
        </Form.Item>

        <Form.Item>
          <Button htmlType="submit" type="primary" icon={<CheckOutlined />} />
        </Form.Item>
      </Space.Compact>
    </Form>
  )
}

interface ApiSettingFormProps {
  code: string
  onSubmit: (data: ApiAuthConfig) => void
  onCancel: () => void
}

const ApiSettingForm = ({ code, onSubmit, onCancel }: ApiSettingFormProps) => {
  const [messageApi, contextHolder] = message.useMessage()

  const [form] = Form.useForm()

  return (
    <>
      {contextHolder}

      <Form
        form={form}
        initialValues={initApiConfigFormValues}
        onFinish={(values: ApiAuthConfigForm) => {
          const apiAuthConfig: ApiAuthConfig = {
            ...values,
            type: 'API',
            api: {
              ...values.api,
              upload: {
                ...values.api.upload,
                max_size: values.api.upload.max_size!,
                other_body: values.api.upload.other_body
                  ? JSON.parse(values.api.upload.other_body)
                  : null,
              },
            },
          }
          onSubmit(apiAuthConfig)
        }}
        onFinishFailed={() => messageApi.error('一些配置项有错误，请检查')}
      >
        <ApiSetting code={code} />

        <Form.Item
          style={{
            display: 'flex',
            justifyContent: 'end',
            marginBottom: '2rem',
          }}
        >
          <Space>
            <Button
              key="cancel"
              type="default"
              onClick={() => {
                form.resetFields()
                onCancel()
              }}
            >
              取消
            </Button>
            <Button key="submit" type="primary" htmlType="submit">
              确认
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </>
  )
}

interface AddCustomProps {
  show?: boolean
  onOk: (code: string, apiAuthConfig: ApiAuthConfig) => Promise<void>
  onCancel: () => void
}

const CODE_REGEX = /^\w+$/

const AddCustom = ({ show, onOk, onCancel }: AddCustomProps) => {
  const [code, setCode] = useState<ManagerCodeState>({
    checked: false,
    code: '',
  })

  useEffect(() => {
    setCode({ checked: false, code: '' })
  }, [show])

  return (
    <div>
      <Modal
        title="添加图床"
        destroyOnClose
        open={show}
        onCancel={onCancel}
        maskClosable={false}
        footer={
          code.checked
            ? []
            : [
              <Button key="cancel" type="default" onClick={onCancel}>
                取消
              </Button>,
            ]
        }
      >
        {code.checked ? (
          <ApiSettingForm
            code={code.code}
            onCancel={onCancel}
            onSubmit={(data) => {
              console.log(initApiConfigFormValues)
              console.log(data)
              newCustomManager('CUSTOM-' + code.code, data)

              onOk(code.code, data)
            }}
          />
        ) : (
          <NewApiCodeForm code={code} setCode={setCode} />
        )}
      </Modal>
    </div>
  )
}

export default AddCustom
