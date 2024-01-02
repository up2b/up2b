import React, { useState } from 'react'
import { Input, InputNumber, Switch, Select, Space } from 'antd'

interface SuccessFieldProps {
  value?: string | number | boolean
  disabled: boolean
  onChange?: (value: string | number | boolean) => void
}

const Status = ({ value, disabled, onChange }: SuccessFieldProps) => {
  const [selected, setSelected] = useState<'string' | 'number' | 'boolean'>(
    typeof value as 'string' | 'number' | 'boolean',
  )

  const render = () => {
    switch (selected) {
      case 'string':
        return (
          <Input
            disabled={disabled}
            onChange={(e) => onChange?.(e.target.value)}
          />
        )
      case 'number':
        return (
          <InputNumber disabled={disabled} onChange={(v) => onChange?.(v!)} />
        )
      case 'boolean':
        return (
          <Switch
            disabled={disabled}
            checked={value as boolean | undefined}
            onChange={onChange}
          />
        )
    }
  }

  return (
    <Space>
      <Select value={selected} disabled={disabled} onChange={setSelected}>
        <Select.Option value="boolean">布尔</Select.Option>
        <Select.Option value="number">数字</Select.Option>
        <Select.Option value="string">字符串</Select.Option>
      </Select>

      {render()}
    </Space>
  )
}

export default Status
