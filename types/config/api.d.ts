interface AuthHeaderMethod {
  type: 'HEADER'
  key?: string
  prefix?: string
}

interface AuthBodyMethod {
  type: 'BODY'
  key: string
}

type AuthMethod = AuthHeaderMethod | AuthBodyMethod

interface ApiAuthConfig {
  type: 'API'
  token: string
  api: ApiConfig
}

interface ApiConfig {
  base_url: string
  auth_method: AuthMethod
  list: ApiListConfig
  delete: ApiDeleteConfig
  upload: ApiUploadConfig
}

interface ApiListGetMethod {
  type: 'GET'
}

interface ApiListPostMethod {
  type: 'POST'
  // TODO: 其他字段以后再补充
}

interface ApiListController {
  items_key: string
  image_url_key: string
  deleted_id_key: string
  thumb_key?: string
}

interface ApiListConfig {
  path: string
  method: ApiListGetMethod | ApiListPostMethod
  controller: ApiListController
}

interface ApiDeletePathKind {
  type: 'PATH'
}

interface ApiDeleteQueryKind {
  type: 'QUERY'
  key: string
}

interface ApiDeleteGetMethod {
  type: 'GET'
  kind: ApiDeletePathKind | ApiDeleteQueryKind
}

interface ApiDeleteDeleteMethod {
  type: 'DELETE'
  kind: ApiDeletePathKind | ApiDeleteQueryKind
}

interface ApiDeletePostMethod {
  type: 'POST'
}

type ApiDeleteMethod =
  | ApiDeleteGetMethod
  | ApiDeletePostMethod
  | ApiDeleteDeleteMethod

interface ApiDeleteJsonController {
  type: 'JSON'
  key: string
  message_key?: string
  should_be: string | number | boolean
}

interface ApiDeleteStatusController {
  type: 'STATUS'
}

type ApiDeleteController = ApiDeleteJsonController | ApiDeleteStatusController

interface ApiDeleteConfig {
  path: string
  method: ApiDeleteMethod
  controller: ApiDeleteController
}

interface ApiUploadJsonContentType {
  type: 'JSON'
  key: string
}

type FileKind = 'STREAM' | 'BUFFER'

interface ApiUploadMultipartContentType {
  type: 'MULTIPART'
  file_kind: FileKind
  file_part_name: string
}

type ApiUploadContentType =
  | ApiUploadJsonContentType
  | ApiUploadMultipartContentType

interface ApiUploadController {
  image_url_key: string
  deleted_id_key: string
  thumb_key?: string
}

interface ApiUploadConfig {
  path: string
  max_size: number
  timeout: number | null
  allowed_formats: string[]
  content_type: ApiUploadContentType
  controller: ApiUploadController
  other_body?: Record<string, any>
}

interface ApiUploadConfigForm extends Omit<ApiUploadConfig, ' other_body'> {
  other_body?: string
}

interface ApiConfigForm {
  base_url: string
  auth_method: AuthMethod
  list: ApiListConfig
  delete: ApiDeleteConfig
  upload: ApiUploadConfigForm
}

interface ApiAuthConfigForm extends Omit<ApiAuthConfig, 'api'> {
  api: ApiConfigForm
}
