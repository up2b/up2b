import React from 'antd'
import { Form, Input, Radio, Space, InputNumber, Select } from 'antd'
import type { FormRule } from 'antd'

const ALLOWED_FORMATS = ['PNG', 'JPEG', 'GIF', 'WEBP', 'BMP']

interface UploadProps {
  data: ApiAuthConfig
  rules: FormRule[]
  urlRules: FormRule[]
  disabled: boolean
  handleChange: (data: ApiAuthConfig) => void
}

const Upload = ({
  data,
  rules,
  urlRules,
  disabled,
  handleChange,
}: UploadProps) => {
  const name = (...key: string[]) => ['api', 'upload', ...key]

  const filteredFormats = ALLOWED_FORMATS.filter(
    (o) => !data.api.upload.allowed_formats.includes(o),
  )

  return (
    <>
      <Form.Item label="接口" name={name('url')} rules={urlRules}>
        <Input
          placeholder="输入上传图片接口"
          value={data.api.upload.url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                upload: {
                  ...data.api.upload,
                  url: e.target.value,
                },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="最大体积" name={name('max_size')} rules={rules}>
        <InputNumber
          placeholder="输入允许的最大体积"
          value={
            data.api.upload.max_size
              ? data.api.upload.max_size / 1024 / 1024
              : undefined
          }
          disabled={disabled}
          addonAfter="MB"
          onChange={(v) =>
            v &&
            handleChange({
              ...data,
              api: {
                ...data.api,
                upload: {
                  ...data.api.upload,
                  max_size: v,
                },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="超时时间" name={name('timeout')}>
        <InputNumber min={0} addonAfter="秒" />
      </Form.Item>

      <Form.Item
        label="允许的格式"
        name={name('allowed_formats')}
        // initialValue={['PNG', 'JPEG', 'GIF']}
        rules={rules}
      >
        <Select
          mode="multiple"
          allowClear
          placeholder="选择图片格式"
          value={data.api.upload.allowed_formats}
          onChange={(values) => {
            handleChange({
              ...data,
              api: {
                ...data.api,
                upload: {
                  ...data.api.upload,
                  allowed_formats: values,
                },
              },
            })
          }}
          options={filteredFormats.map((item) => ({
            value: item,
            label: item,
          }))}
          disabled={disabled}
        />
      </Form.Item>

      <Form.Item name={name('content_type', 'type')} label="请求体类型">
        <Radio.Group
          value={data.api.upload.content_type.type}
          disabled={disabled}
          onChange={(e) => {
            handleChange({
              ...data,
              api: {
                ...data.api,
                upload: {
                  ...data.api.upload,
                  content_type:
                    e.target.value === 'JSON'
                      ? {
                        ...(data.api.upload
                          .content_type as ApiUploadJsonContentType),
                        type: 'JSON',
                      }
                      : {
                        ...(data.api.upload
                          .content_type as ApiUploadMultipartContentType),
                        type: 'MULTIPART',
                      },
                },
              },
            })
          }}
        >
          <Radio value="MULTIPART">multipart</Radio>
          <Radio value="JSON">json</Radio>
        </Radio.Group>
      </Form.Item>

      {data.api.upload.content_type.type === 'MULTIPART' ? (
        <Space wrap>
          <Form.Item
            name={name('content_type', 'file_kind')}
            label="上传类型"
            tooltip="流式响应支持上传进度，流式上传失败时可尝试更换为 bytes"
          >
            <Radio.Group
              value={data.api.upload.content_type.file_kind ?? 'STREAM'}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    upload: {
                      ...data.api.upload,
                      content_type: {
                        ...(data.api.upload
                          .content_type as ApiUploadMultipartContentType),
                        file_kind: e.target.value,
                      },
                    },
                  },
                })
              }}
            >
              <Radio value="STREAM">流</Radio>
              <Radio value="BUFFER">bytes</Radio>
            </Radio.Group>
          </Form.Item>

          <Form.Item
            label="图片的表单键"
            name={name('content_type', 'file_part_name')}
            rules={rules}
          >
            <Input
              value={data.api.upload.content_type.file_part_name}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    upload: {
                      ...data.api.upload,
                      content_type: {
                        ...(data.api.upload
                          .content_type as ApiUploadMultipartContentType),
                        file_part_name: e.target.value,
                      },
                    },
                  },
                })
              }}
            />
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
            <Input value={data.api.upload.content_type.key} />
          </Form.Item>

          <Form.Item
            name={name('content_type', 'other_body')}
            label="除图片之外的其他 json 数据"
          >
            <Input.TextArea />
          </Form.Item>
        </>
      )}

      <Space wrap>
        <Form.Item
          label="图片键"
          name={name('controller', 'image_url_key')}
          rules={rules}
        >
          <Input
            value={data.api.upload.controller.image_url_key}
            disabled={disabled}
            onChange={(e) => {
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    controller: {
                      ...data.api.upload.controller,
                      image_url_key: e.target.value,
                    },
                  },
                },
              })
            }}
          />
        </Form.Item>

        <Form.Item
          label="删除 id 键"
          name={name('controller', 'deleted_id_key')}
          rules={rules}
        >
          <Input
            value={data.api.upload.controller.deleted_id_key}
            disabled={disabled}
            onChange={(e) => {
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    controller: {
                      ...data.api.upload.controller,
                      deleted_id_key: e.target.value,
                    },
                  },
                },
              })
            }}
          />
        </Form.Item>

        <Form.Item name={name('controller', 'thumb_key')} label="图片缓存键">
          <Input
            value={data.api.upload.controller.thumb_key}
            disabled={disabled}
            onChange={(e) => {
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    controller: {
                      ...data.api.upload.controller,
                      thumb_key: e.target.value,
                    },
                  },
                },
              })
            }}
          />
        </Form.Item>
      </Space>
    </>
  )
}

export default Upload
