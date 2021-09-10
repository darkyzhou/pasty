# pasty

使用 Rocket 和 RocksDB 实现的最简单的 Pastebin/短链接 服务。其中 RocksDB 提供数据持久化功能，在理论上能够提供高速的键值访问，以及承受程序的突然崩溃和机器的突然断电等灾害，同时保证已有的数据不会冲突。

## 配置

默认的配置在 `Rocket.toml` 中：

```toml
[default]
address = "0.0.0.0"

[default.limits]
# 默认只允许存储大小至多 128 KiB 的纯文本或链接
"plain/text" = "128 KiB"

[default.pasty]
# 数据库文件夹的目录，默认为 data
db_path = "data"

# 添加链接或纯文本时需要的访问密码，如果不需要可以设为空，即 ""
access_password = "password"

# 随机产生的链接长度
random_id_length = 6

# 首页显示的文本，如果指定了下面的 index_link，那么会直接重定向到对应的链接
index_text = "欢迎使用 Pasty！具体的用法请参考：https://github.com/darkyzhou/pasty"
index_link = ""
```

## 用法

下面的例子里假设服务器架设在 `https://pasty.dev` 上。

### 添加链接（指定名称）

下面的例子存储 `https://www.bilibili.com/video/BV1GJ411x7h7` 作为短链接 `rickroll`，使用密码 `12345`。密码仅供后续删除、更新短链接使用，访问短链接不需要密码。如果在配置文件里设置了密码，那么创建链接时需要使用 `access` 指定：

```
curl --data 'https://www.bilibili.com/video/BV1GJ411x7h7' 'https://pasty.dev/rickroll?type=link&pwd=12345&access=password'
```

存储之后，其他用户使用浏览器访问 `https://pasty.dev/rickroll` 就会被 301 到 Rick Roll 视频。

> 因为链接是默认类型，所以我们可以省略 `type=link`。

### 添加链接（使用随机名称）

如果想让 Pasty 随机生成一个链接名称，可以省略上面的 `rickroll`：

```
curl --data '...' 'https://pasty.dev/?type=link&pwd=12345&access=password'
```

> 因为链接是默认类型，所以我们可以省略 `type=link`。

### 添加纯文本

存储 `Never gonna give you up` 作为短链接 `rick` 指向的纯文本内容，密码为 `12345`。存储之后，其他用户使用浏览器访问 `https://pasty.dev/rick` 时会显示上述纯文本。

```
curl --data 'Never gonna give you up' 'https://pasty.dev/rick?type=plain&pwd=12345&access=password'
```

### 查看链接或纯文本访问次数

如下所示可以查看一个链接或纯文本的总访问次数。

```
curl 'https://pasty.dev/rick/stat'
```

### 更新项目

下面的例子使用创建时指定的密码来更新链接的类型或者内容，例如将上面的 `rick` 指向的纯文本修改为 `Never gonna let you down`。同样，如果在配置文件里配置了密码，需要使用 `access` 来指定。

```
curl --data 'Never gonna let you down' 'https://pasty.dev/rick?type=plain&pwd=12345&access=password'
```

### 删除项目

使用创建时指定的密码来删除链接：

```
curl -X DELETE 'https://pasty.dev/rick?pwd=12345'
```

## 部署

使用 docker 进行部署是最简单的方式：

```
docker run -d --restart=unless-stopped -p 8000:8000 darkyzhou/pasty
```

下面的例子里将当前目录的 `Rocket.toml` 映射到容器里使得自定义的配置生效。

```
docker run -d --restart=unless-stopped -v ./Rocket.toml:/Rocket.toml -p 8000:8000 darkyzhou/pasty
```

## TODO

- [ ] 测试
- [ ] 其他功能
