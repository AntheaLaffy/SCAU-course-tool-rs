# 华南农业大学教务系统登录 API 文档

> 基于正方教务系统 V9.0（SCAU 专用版）

## 基础信息

- **Base URL**: `https://jwzf.scau.edu.cn`
- **编码**: UTF-8

---

## 1. 获取 CSRF Token

### 接口

```
GET /jwglxt/xtgl/login_slogin.html
```

### 说明

从登录页面 HTML 中提取 `csrftoken`：

```html
<input type="hidden" id="csrftoken" value="uuid1,uuid2">
```

### 重要提示

- `value` 格式为 `uuid1,uuid2`
- **只需要逗号前的部分**（即第一个 UUID）
- 示例：`174d03f8-47bf-4837-8cdf-ff98ccc6882d`

---

## 2. 获取 RSA 公钥

### 接口

```
GET /jwglxt/xtgl/login_getPublicKey.html?time={timestamp}
```

### URL 示例

```
https://jwzf.scau.edu.cn/jwglxt/xtgl/login_getPublicKey.html?time=1737025800000
```

### 响应格式

```json
{
  "modulus": "AJBbodktkoJcWjhpsJW8qpqMKXhuqH\\/BFvOl8WtI\\/21TJ08lN9GoaxBadJH2qyWLZoPBD6tyBzh2xQzSrp1iv+Wp0\\/TXuDzPd7el0VY0Jf3hFka9JxpsztFgIIsnG+x7BMXpnFlbjq59Od\\/49CV+0K8DSL5Rjxy1Ol9YL8Ufb0Wz",
  "exponent": "AQAB"
}
```

### 关键说明

| 字段 | 说明 | 格式 |
|------|------|------|
| `modulus` | RSA 模数 | Base64 + URL转义（`\/`） |
| `exponent` | RSA 指数 | Base64（通常为 `AQAB`） |

### ⚠️ 重要：格式处理

**正方教务系统返回的是 Base64 编码，不是 Hex！**

```rust
// 错误写法：使用 hex::decode()
// 正确写法：
let modulus_unescaped = modulus.replace("\\/", "/");  // 先处理 URL 转义
let modulus_bytes = base64::decode(&modulus_unescaped).unwrap();
```

---

## 3. 用户登录

### 接口

```
POST /jwglxt/xtgl/login_slogin.html
```

### URL 示例

```
https://jwzf.scau.edu.cn/jwglxt/xtgl/login_slogin.html
```

### 请求头

```
Content-Type: application/x-www-form-urlencoded
Referer: https://jwzf.scau.edu.cn/jwglxt/xtgl/login_slogin.html
```

### 请求体参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `csrftoken` | string | 是 | 从页面获取的 CSRF Token（逗号前部分） |
| `language` | string | 是 | 固定值 `zh_CN` |
| `yhm` | string | 是 | 用户名（学号） |
| `mm` | string | 是 | RSA 加密后的密码（Base64 编码） |

### 请求体示例

```
csrftoken=174d03f8-47bf-4837-8cdf-ff98ccc6882d&language=zh_CN&yhm=202425910219&mm=加密后的密码
```

### 响应行为

⚠️ **登录成功返回 302 跳转，不是 JSON！**

- **成功**：HTTP 302 → 重定向到 `index_initMenu.html` 或 `xtgl/index.html`
- **失败**：返回登录页面 HTML（可检查 "用户名或密码不正确"）

### 判断登录成功

```rust
let final_url = response.url().to_string();
if final_url.contains("index_initMenu.html") || final_url.contains("xtgl/index.html") {
    // 登录成功
}
```

---

## 4. 密码 RSA 加密流程

### 步骤

1. **获取公钥**: 调用 `login_getPublicKey.html` 获取 `modulus` 和 `exponent`
2. **处理转义**: 将 `\/` 替换为 `/`（JSON 中的 URL 转义）
3. **Base64 解码**: 解码 `modulus` 和 `exponent` 为字节
4. **构建 BigInt**: 使用 `BigUint::from_bytes_be()` 转换
5. **构建 RSA 公钥**: 使用 modulus 和 exponent 创建 RSA 公钥
6. **加密密码**: 使用 PKCS1v15 填充方式加密原始密码
7. **Base64 编码**: 将加密结果进行 Base64 编码

### JavaScript 参考实现（正方系统前端）

```javascript
var rsa = new RSAKey();
rsa.setPublic(modulus, exponent);
var encrypted = rsa.encrypt(password);
// 结果为 Base64 编码的字符串
```

### Rust 实现参考

```rust
use rsa::{RsaPublicKey, Pkcs1v15Encrypt};
use num_bigint_dig::BigUint;
use base64::{Engine, Engine as _};

// 1. 处理 URL 转义
let modulus_unescaped = modulus.replace("\\/", "/");
let exponent_unescaped = exponent.replace("\\/", "/");

// 2. Base64 解码
let modulus_bytes = base64::prelude::BASE64_STANDARD.decode(&modulus_unescaped).unwrap();
let exponent_bytes = base64::prelude::BASE64_STANDARD.decode(&exponent_unescaped).unwrap();

// 3. 构建 BigInt
let modulus = BigUint::from_bytes_be(&modulus_bytes);
let exponent = BigUint::from_bytes_be(&exponent_bytes);

// 4. 构建 RSA 公钥
let public_key = RsaPublicKey::new(modulus, exponent).unwrap();

// 5. 加密（PKCS1v15）
let padding = Pkcs1v15Encrypt;
let encrypted = public_key.encrypt(&mut OsRng, padding, password.as_bytes()).unwrap();

// 6. Base64 编码结果
let result = base64::Engine::encode(&base64::prelude::BASE64_STANDARD, &encrypted);
```

---

## 5. 完整登录流程

```
1. 获取 CSRF Token
   GET /jwglxt/xtgl/login_slogin.html
   → 从 HTML 中提取 csrftoken（取逗号前的 UUID）

2. 获取 RSA 公钥
   GET /jwglxt/xtgl/login_getPublicKey.html?time={timestamp}
   → 解析 modulus 和 exponent（Base64 + URL 转义）

3. 加密密码
   → 使用 RSA PKCS1v15 加密 → Base64 编码

4. 提交登录请求
   POST /jwglxt/xtgl/login_slogin.html
   → Content-Type: application/x-www-form-urlencoded
   → Referer: 必须设置
   → 检测最终 URL 是否包含 index_initMenu.html

5. 保存 Cookie（JSESSIONID, route）
   → 用于后续请求
```

---

## 6. 关键 Cookie

| Cookie | 说明 |
|--------|------|
| `JSESSIONID` | 服务器会话 ID（登录前获取，登录后保持） |
| `route` | 路由标识（登录前获取） |

---

## 7. 常见错误

| 现象 | 原因 | 解决方案 |
|------|------|----------|
| "Invalid character" | RSA 公钥格式错误 | 确认使用 Base64 解码，非 Hex |
| 返回登录页面 | CSRF Token 过期或密码错误 | 重新获取 Token，检查密码 |
| 解析 JSON 失败 | 期望 JSON 但收到 HTML | 检查是否处理了 302 跳转 |
| RSA 加密失败 | 公钥参数无效 | 确认 modulus/exponent 长度正确 |

---

## 8. 注意事项

1. **CSRF Token**: 每次访问登录页面会生成新的 token，必须先获取再登录
2. **Token 格式**: HTML 中可能是 `uuid1,uuid2`，只需取逗号前部分
3. **时间戳**: 获取公钥时需要带毫秒级时间戳参数
4. **Cookie**: 登录前需先访问页面获取 Cookie（route, JSESSIONID）
5. **Referer**: POST 登录请求需要设置正确的 Referer 头
6. **编码**: 所有请求和响应使用 UTF-8 编码
