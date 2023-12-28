import React, { useEffect, useState } from 'react'
import { Button, Form, Input, Modal, Space } from 'antd'
import ApiSetting from './api'
import { CheckOutlined } from '@ant-design/icons'
import { newCustomManager } from '~/lib'

interface AddCustomProps {
  show?: boolean
  onOk: () => void
  onCancel: () => void
}

const CODE_REGEX = /^\w+$/

const AddCustom = ({ show, onOk, onCancel }: AddCustomProps) => {
  const [code, setCode] = useState<{
    checked: boolean
    code: string
  }>({ checked: false, code: '' })

  const [authConfig, setAuthConfig] = useState<ApiAuthConfig | null>(null)

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
        footer={[
          <Button key="cancel" type="default" onClick={onCancel}>
            取消
          </Button>,
          <Button
            key="submit"
            type="primary"
            onClick={() => {
              console.log(authConfig)
              newCustomManager('CUSTOM-' + code.code, authConfig!)
              onOk()
            }}
            disabled={!code.checked}
          >
            确认
          </Button>,
        ]}
      >
        {code.checked ? (
          <ApiSetting
            code={code.code}
            onChange={(data) => {
              setAuthConfig(data)
            }}
          />
        ) : (
          <Form>
            <Form.Item
              name="code"
              label="图床代码"
              tooltip="仅支持字母、数字、下划线。通常为图床名称，如smms、imgtg"
              hasFeedback
              rules={[
                { required: true },
                {
                  type: 'string',
                  pattern: CODE_REGEX,
                  warningOnly: true,
                  message: '仅支持字母、数字、下划线',
                },
              ]}
            >
              <Space>
                <Input
                  placeholder="图床代码"
                  value={code.code}
                  onChange={(e) =>
                    setCode((pre) => ({
                      ...pre,
                      code: e.target.value.toUpperCase(),
                    }))
                  }
                />
                <Button
                  htmlType="submit"
                  type="primary"
                  icon={<CheckOutlined />}
                  onClick={() => setCode((pre) => ({ ...pre, checked: true }))}
                  disabled={!CODE_REGEX.test(code.code)}
                />
              </Space>
            </Form.Item>
          </Form>
        )}
      </Modal>
    </div>
  )
}

export default AddCustom
