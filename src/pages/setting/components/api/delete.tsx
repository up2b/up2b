import React from 'react'
import { Form, Input, Radio, Space } from 'antd'
import type { FormRule } from 'antd'
import Status from '../status'

interface DeleteProps {
  rules: FormRule[]
  pathRules: FormRule[]
  disabled: boolean
}

const Delete = ({ rules, pathRules, disabled }: DeleteProps) => {
  const name = (...key: string[]) => ['api', 'delete', ...key]

  const form = Form.useFormInstance()
  const methodTypeValue = Form.useWatch(
    name('method', 'type'),
    form,
  ) as ApiDeleteMethod['type']
  const methodKindTypeValue = Form.useWatch(
    name('method', 'kind', 'type'),
    form,
  ) as (ApiDeletePathKind | ApiDeleteQueryKind)['type']
  const controllerTypeValue = Form.useWatch(
    name('controller', 'type'),
    form,
  ) as ApiDeleteController['type']

  return (
    <>
      <Form.Item label="路径" name={name('path')} rules={pathRules}>
        <Input placeholder="输入图片删除接口路径" disabled={disabled} />
      </Form.Item>

      <Form.Item label="请求方法" name={name('method', 'type')}>
        <Radio.Group disabled={disabled}>
          <Radio value="DELETE">DELETE</Radio>
          <Radio value="GET">GET</Radio>
          <Radio value="POST">POST</Radio>
        </Radio.Group>
      </Form.Item>

      {methodTypeValue === 'GET' || methodTypeValue === 'DELETE' ? (
        <>
          <Form.Item
            name={name('method', 'kind', 'type')}
            label="删除 id 所在位置"
          >
            <Radio.Group disabled={disabled}>
              <Radio value="PATH">路径</Radio>
              <Radio value="QUERY">查询参数</Radio>
            </Radio.Group>
          </Form.Item>

          {methodKindTypeValue === 'QUERY' ? (
            <Form.Item name={name('method', 'kind', 'key')} label="key">
              <Input />
            </Form.Item>
          ) : null}
        </>
      ) : null}

      <Form.Item
        name={name('controller', 'type')}
        label="验证途径"
        tooltip="删除图片如果有响应体则应该传入响应体中的有效属性键，否则以响应状态码判断是否成功"
      >
        <Radio.Group disabled={disabled}>
          <Radio value="JSON">json响应</Radio>
          <Radio value="STATUS">状态码</Radio>
        </Radio.Group>
      </Form.Item>

      {controllerTypeValue === 'JSON' ? (
        <Form.Item>
          <Space wrap>
            <Form.Item
              label="成功状态键"
              name={name('controller', 'key')}
              rules={rules}
            >
              <Input placeholder="删除成功与否的键名" disabled={disabled} />
            </Form.Item>

            <Form.Item
              label="成功状态值"
              tooltip="删除成功时值应该是什么"
              name={name('controller', 'should_be')}
              rules={rules}
            >
              <Status disabled={disabled} />
            </Form.Item>

            <Form.Item
              label="失败消息键"
              name={name('controller', 'message_key')}
              rules={rules}
            >
              <Input placeholder="删除失败的消息键名" disabled={disabled} />
            </Form.Item>
          </Space>
        </Form.Item>
      ) : null}
    </>
  )
}

export default Delete
