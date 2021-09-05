# pasty

使用 Rocket 和 RocksDB 实现的最简单的 pastebin 服务。其中 RocksDB 提供数据持久化功能，能够承受程序的突然崩溃、机器的突然断电等灾害，同时保证已有的数据不会冲突。

## 用法

下面的例子里假设服务器架设在 `https://pasty.dev` 上。

### 添加链接

存储 `https://www.bilibili.com/video/BV1GJ411x7h7` 作为短链接 `rickroll`，使用密码 `12345`。密码仅供后续删除、更新短链接使用，访问短链接不需要密码。存储之后，其他用户使用浏览器访问 `https://pasty.dev/rickroll` 就会被 301 到 Rick Roll 视频。

```
curl -X POST --data 'https://www.bilibili.com/video/BV1GJ411x7h7' https://pasty.dev/rickroll?type=link&pwd=12345
```

### 添加纯文本

存储 `Never gonna give you up` 作为短链接 `rick` 指向的纯文本内容，密码为 `12345`。存储之后，其他用户使用浏览器访问 `https://pasty.dev/rick` 时会显示上述纯文本。

```
curl -X POST --data 'Never gonna give you up' https://pasty.dev/rick?type=plain&pwd=12345
```

### 更新项目

使用创建时指定的密码来更新链接的类型或者内容，例如将上面的 `rick` 指向的纯文本修改为 `Never gonna let you down`：

```
curl -X POST --data 'Never gonna let you down' https://pasty.dev/rick?type=plain&pwd=12345
```

### 删除项目

使用创建时指定的密码来删除链接：

```
curl -X DELETE https://pasty.dev/rick?pwd=12345
```

## 部署

## TODO

- [ ] 测试
- [ ] 其他功能
