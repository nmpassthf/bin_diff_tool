# Bin diff tool

二进制文件增量更新工具

## 功能

- 支持对比两个目录，生成二进制文件差异补丁包
- 应用更新补丁包到目标目录，生成更新后的目录
- 支持大文件处理，内存占用低

## 特性

无系统依赖和系统内程序依赖, 纯 Rust 实现. 可跨平台使用 (Windows, Linux, MacOS).

## 使用方法

`dft diff <source_dir> <target_dir> -o patch_archive.tgz` 生成补丁包
`dft apply <target_dir> -p patch_archive.tgz` 应用补丁包 (更新目标目录)
`dft append <patch_version_first.tgz> <patch_version_second.tgz> -o combined_patch.tgz` 合并两个补丁包, 有版本依赖关系

`dft show <patch_archive.tgz>` 显示补丁包内容 - 列出新增、删除、修改的文件列表 (只对文本显示修改内容, 所有二进制文件均使用替换方式)

## 补丁包结构

补丁包为 tar.gz 格式，包含以下内容：
- `added/` 目录：新增文件
- `deleted/` 目录：删除文件列表
- `modified/` 目录：修改文件的差异数据
- `metadata.toml` 文件：补丁包元数据，包含版本信息、生成时间等
- `checksums.toml` 文件：补丁包内文件的校验和信息

