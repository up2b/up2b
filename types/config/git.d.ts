interface GitAuthConfig {
  type: 'GIT'
  base_url: string
  token: string
  username: string
  repository: string
  // 保存目录,默认为 up2b
  path?: string
}

type GithubAuthConfig = Omit<GitAuthConfig, 'base_url'>
