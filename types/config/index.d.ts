type ManagerKind = 'API' | 'CHEVERETO'

type ManagerCode = 'SMMS' | 'IMGSE' | 'IMGTG'

type _APIKey<T extends ManagerCode> = T extends 'SMMS' ? T : never

type APIManagerKey = _APIKey<ManagerCode>

type _CheveretoKey<T extends ManagerCode> = T extends 'IMGSE' | 'IMGTG'
  ? T
  : never

type CheveretoManagerKey = _CheveretoKey<ManagerCode>

type InferKeyType<
  T extends ApiAuthConfig['type'] | CheveretoAuthConfig['type'],
> = T extends 'API' ? APIManagerKey : CheveretoManagerKey

type Extra = Record<string, string>

interface CheveretoAuthConfig {
  type: 'CHEVERETO'
  username: string
  password: string
  extra: Extra | null
}

interface ImgseAuthConfig extends CheveretoAuthConfig {
  extra: {
    token?: string
  }
}

interface BaseProxy {
  host?: string
  port?: number
}

interface HttpProxy extends BaseProxy {
  type: 'http'
}

interface HttpsProxy extends BaseProxy {
  type: 'https'
}

interface Socks5Proxy extends BaseProxy {
  type: 'socks5'
}

interface Socks5hProxy extends BaseProxy {
  type: 'socks5h'
}

type Proxy = HttpProxy | HttpsProxy | Socks5Proxy | Socks5hProxy

type InferAuthConfigKind<K extends ManagerCode> = K extends APIManagerKey
  ? ApiAuthConfig
  : K extends CheveretoManagerKey
  ? CheveretoAuthConfig
  : never

type AuthConfigKinds = {
  [K in ManagerCode]?: InferAuthConfigKind<K>
}

interface Config {
  using: ManagerCode
  use_proxy: boolean
  automatic_compression: boolean
  proxy?: Proxy
  auth_config: AuthConfigKinds
}

type ProxyProtocol = 'http' | 'https' | 'socks5' | 'socks5h'

interface ManagerItem {
  type: ManagerKind
  name: string
  key: string
  index?: string
}
