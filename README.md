# swc-plugin-another-transform-imports

Another wasm plugin for swc, inspired from [babel-plugin-transform-imports](https://www.npmjs.com/package/babel-plugin-transform-imports).

## Installation

```bash
npm install --save-dev swc-plugin-another-transform-imports
# or
yarn add -D swc-plugin-another-transform-imports
```

## Usage

It follows resolving rule of node.js, can be use in `.swcrc`，`webpack.config.js` or `next.config.js` and so on. For Example:

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-another-transform-imports",
          {
            "antd": {
              "transform": "antd/lib/${member}",
              "skipDefaultConversion": false,
              "preventFullImport": true,
              "style": "antd/lib/${member}/style",
              "memberTransformers": ["dashed_case"]
            },
            "lodash": {
              "transform": "lodash/${member}",
              "preventFullImport": true
            }
          }
        ]
      ]
    }
  }
}
```

Can convert the following lines:

```js
import { Button as MyButton, BackTop } from "antd";
import { merge } from "lodash";
```

To:

```js
import MyButton from "antd/lib/button";
import BackTop from "antd/lib/back-top";
import "antd/lib/back-top/style";
import "antd/lib/button/style";
import merge from "lodash/merge";
```

### For next.js users

**Ensure you read [next.js compiler docs](https://nextjs.org/docs/advanced-features/compiler) first!!!!**

Since the semantic version association of `@swc/core` (npm) and `swc_core` (rust) for next.js is still experimental. next.js has a lot of breaking change in the swc native plugin mechanism between major and even minor versions.

It is possible that as next.js and swc are updated, the current plugin will fail in the new version. Hopefully the new plugin api for swc will be stable soon. Here is the current version correspondence.

| next.js versions                     | swc_versions       | This package version                                                                                                                                         |
|--------------------------------------|-------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 12.3.x                               | -                 | 0.1.5                                                                                                                                                        |
| 13.0.x                               | -                 | 0.2.1                                                                                                                                                        |
| 13.2.4 ~ 13.3.1                      | -                 | not support [https://github.com/vercel/next.js/issues/46989#issuecomment-1486989081](https://github.com/vercel/next.js/issues/46989#issuecomment-1486989081) |
| 13.3.1 ~ 13.4.3                      | -                 | 0.2.3                                                                                                                                                        |
| 13.4.3-canary.2 ~ 13.4.7             | 1.3.58 ~ 1.3.62   | 0.2.4                                                                                                                                                        |
| 13.4.8 ~ v13.4.10-canary.0           | 1.3.63 ~ 1.3.67   | 0.2.5                                                                                                                                                        |
| 13.4.10-canary.1 ~                   | 1.3.68 ~ 1.3.80   | 0.2.6                                                                                                                                                        |
| ~ 14.1.0                             | 1.3.81 ~ 1.3.105  | 0.2.7                                                                                                                                                        |
| -                                    | 1.3.106 ~ 1.3.107 | 0.2.8                                                                                                                                                        |
| 14.1.1-canary.52 ~ 15.0.0-canary.28  | 1.4.x             | 0.2.9                                                                                                                                                        |
| 15.0.0-canary.29 ~ 15.0.0-canary.36  | 1.5.x             | 1.5.0                                                                                                                                                        |
| 15.0.0-canary.37 ~ 15.0.0-canary.116 | 1.6.x             | 1.6.0                                                                                                                                                        |
| 15.0.0-canary.117 ~ v15.0.0-canary.183                 | 1.7.0 ~ 1.7.27             | 1.7.0  |
| 15.0.0 ~ 15.0.2  | 1.7.28 ~ 1.7.42  | 1.7.42 |
| -  | 1.8.0  | 1.8.0 |
| 15.0.3 ~ 15.1.7 | 1.9.0 ~ 1.9.2 | 1.9.0 |
| - | 1.9.3 | 1.9.3 |
| 15.2.0 | 1.10.0 ~ 1.10.18 | 1.10.0 |
| 15.2.1 ~ 15.3.5 | 1.11.1 ~ 1.11.31 | 1.11.1 |
| 15.4.0 ~ 15.4.3 | 1.12.0 ~ 1.12.14 | 1.12.0 |
| 15.4.4 ~ 15.5.6 | 1.13.0 ~ 1.13.5  | 1.13.0 |
| - | 1.14.0 | 1.14.0 |
| - | 1.15.0 ~ 1.15.3 | 1.15.0 | 

[@swc/core and swc_core version mappings](https://swc.rs/docs/plugin/selecting-swc-core)

### For antd users

`antd/es/xxx/style` and `antd/lib/xxx/style` will introduce `less`, please add `less-loader` by yourself.

If you use antd and next.js at the same time, it will be a bit troublesome to work with them, you can use `next-plugin-antd-less` for convenience, see [issue 1](https://github.com/lonelyhentai/swc-plugin-another-transform-imports/issues/1) for an example.

## Options

| Name                    | Type                       | Required | Default     | Description                                                                                                                                     |
|-------------------------|----------------------------|----------|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| `transform`             | `string`                   | yes      | `undefined` | The library name to use instead of the one specified in the import statement. ${member} will be replaced with the member, aka Grid/Row/Col/etc. |
| `preventFullImport`     | `boolean`                  | no       | `true`      | Whether or not to throw when an import is encountered which would cause the entire module to be imported.                                       |
| `skipDefaultConversion` | `boolean`                  | no       | `false`     | When set to true, will preserve `import { X }` syntax instead of converting to `import X`.                                                      |
| `style`                 | `string`                   | no       | `false`     | The style path of the member, ${member} will be replaced with the member, aka Grid/Row/Col/etc.                                                 |
| `memberTransformers`    | `Array<MemberTransformer>` | no       | []          | Member transformers                                                                                                                             |

### 1. `type MemberTransformer`

```typescript
type MemberTransformer =
  | "camel_case"
  | "kebab_case"
  | "pascal_case"
  | "snake_case"
  | "upper_case"
  | "upper_first"
  | "lower_case"
  | "lower_first"
  | "dashed_case";
```

## Common Issues

Usually upgrading to the latest version of @swc/core and other swc tools will solve the problem, see the following issue for typical solution:

1. [plugin-styled-components crashes with 'Error while importing "env"."\_\_get_transform_plugin_config": unknown import.' ](https://github.com/swc-project/plugins/issues/60)
2. [about next.js and antd](https://github.com/lonelyhentai/swc-plugin-another-transform-imports/issues/1)
3. [about next.js 13+](https://github.com/lonelyhentai/swc-plugin-another-transform-imports/issues/2)

## Fork & Modify

You can simply fork this plugin, modify its source code to suit your custom needs. For fast validation, you don't necessarily have to publish the modified project, but simply require your wasm file to the project. For example:

```bash
cargo prepublish
```

or

```bash
npm run prepublish
```

Then copy your wasm target file, and set your config to:

```js
module.exports = {
  ...,
  "jsc": {
    "experimental": {
      "plugins": [
        [
          require.resolve("./path/to/your-modified-plugin.wasm"),
          {
            ..your options
          }
        ]
      ]
    }
  },
  ...
}
```
