# amc-gitflow-rs

一个 Rust 实现的 Gitflow 增强工具，与 GitHub CLI (`gh`) 深度集成。该工具遵循标准的 Gitflow 规范，但将分支合并与生命周期管理转移到 GitHub 的 PR (Pull Request) 系统中，以利用协作评审、自动化检查和完整的操作审计。

## 前置要求

- **Git**: 已安装并配置。
- **GitHub CLI (`gh`)**: 已安装并通过 `gh auth login` 完成认证。
- **Rust**: 从源码构建时需要。

## GitHub 仓库设置

为确保所有自动化功能（特别是 Release 和 Bugfix 流程）正常运行，请按以下步骤配置仓库：

1.  **标签 (Labels)**:
    *   创建 `release` 标签（用于分类版本发布 PR）。
    *   创建 `bugfix` 标签（用于错误修复 PR）。
    *   创建 `feature` 标签（用于功能开发 PR）。
2.  **分支保护 (Branch Protection)**:
    *   建议对 `master`/`main` 和 `develop` 分支启用保护规则。
    *   建议开启 "Require a pull request before merging"。
3.  **合并设置 (Merge Settings)**:
    *   为了保持提交历史的整洁清晰，建议在 GitHub 仓库设置中启用 **"Allow squash merging"** 或 **"Allow rebase merging"**。
4.  **工作流权限 (Workflow Permissions)**:
    *   `gh` 登录所用的 Token 必须具备 `repo` 和 `workflow` 作用域，以便工具能够创建 PR、创建 Git Tag 以及发布 GitHub Release。

## 安装

```bash
# 克隆仓库并安装到 bin 目录
git clone https://github.com/your-repo/amc-gitflow-rs.git
cd amc-gitflow-rs
cargo install --path .
```

## 工作流指南

### 1. 初始化
在项目根目录运行，配置分支名称与前缀：
```bash
amc-gitflow-rs init
```

### 2. 功能开发 (Feature) Workflow
用于日常新功能开发。
- **开始**：`amc-gitflow-rs feature start <name>` - 从 `develop` 分支切出。
- **发布**：`amc-gitflow-rs feature publish` - 推送分支并创建指向 `develop` 的 PR。
- **完成**：`amc-gitflow-rs feature finish` - 检查 PR 是否已合并，拉取更新并删除本地开发分支。

### 3. 发布版本 (Release) Workflow
用于正式发布生产版本。
- **开始**：`amc-gitflow-rs release start <version>` - 从 `develop` 切出发布分支。
- **发布**：`amc-gitflow-rs release publish` - 推送到所有远程仓库，创建指向 `master` 的 PR，并引导填写包含 Summary, Changelog, Checklist 的发布模板。
- **完成**：`amc-gitflow-rs release finish [--auto]`
    - **第 1 步**: 验证 GitHub PR 已合并。
    - **第 2 步**: 同步 `master` 分支，创建 Git Tag 并推送至所有远程。
    - **第 3 步**: 创建 **GitHub Release**（交互式编辑或使用 `--auto` 自动生成说明）。
    - **第 4 步**: 执行 Back-merge，将 `master` 变更流回 `develop`。
    - **第 5 步**: 清理本地及远程的发布分支。
    - **提示**: `--auto` 参数会跳过所有提问，并自动 bump 项目的小版本号 (Patch)。

### 4. 故障修复 (Bugfix) Workflow
深度集成了 GitHub Issues 流程。
- **开始**：`amc-gitflow-rs bugfix start` - 交互式列出仓库中打开的 Issues 供选择。自动根据 Issue ID 和标题创建分支。
- **发布**：`amc-gitflow-rs bugfix publish` - 创建 PR 并自动标注 "Closes #IssueID"。
- **完成**：`amc-gitflow-rs bugfix finish` - PR 合并后清理分支。

## 配置参考

配置信息存储在项目的本地 `.git/config` 中。

| 参数名 | 说明 | 默认值 |
|-----|-----|---------|
| `amc-gitflow-rs.branch.product` | 生产发布分支 | `master` |
| `amc-gitflow-rs.branch.develop` | 主开发分支 | `develop` |
| `amc-gitflow-rs.prefix.feature` | 功能分支前缀 | `feature/` |
| `amc-gitflow-rs.prefix.release` | 发布分支前缀 | `release/` |
| `amc-gitflow-rs.prefix.versiontag` | 版本标签前缀 | (空) |

管理命令：
```bash
amc-gitflow-rs config list
amc-gitflow-rs config set <key> <value>
```

## 许可证

MIT / Apache 2.0

