# Next.js + Playwright

This example shows how to configure Playwright to work with Next.js.

## Deploy your own

Deploy the example using [Vercel](https://vercel.com?utm_source=github&utm_medium=readme&utm_campaign=next-example) or preview live with [StackBlitz](https://stackblitz.com/github/vercel/next.js/tree/canary/examples/with-playwright)

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https://github.com/vercel/next.js/tree/canary/examples/with-playwright&project-name=with-playwright&repository-name=with-playwright)

## How to use

Execute [`create-next-app`](https://github.com/vercel/next.js/tree/canary/packages/create-next-app) with [npm](https://docs.npmjs.com/cli/init), [Yarn](https://yarnpkg.com/lang/en/docs/cli/create/), or [pnpm](https://pnpm.io) to bootstrap the example:

```bash
npx create-next-app --example with-playwright with-playwright-app
```

```bash
yarn create next-app --example with-playwright with-playwright-app
```

```bash
pnpm create next-app --example with-playwright with-playwright-app
```

Deploy it to the cloud with [Vercel](https://vercel.com/new?utm_source=github&utm_medium=readme&utm_campaign=next-example) ([Documentation](https://nextjs.org/docs/deployment)).

# やったこと

## 準備

```bash
# playwrightを動かすのに必要、ヘッドレスブラウザと思われる
npx playwright install
# hostにライブラリ追加した、playwrightに必要らしい
sudo npx playwright install-deps
```

## nginx(docker)

```bash
# copy config
docker run --rm --entrypoint=cat nginx /etc/nginx/nginx.conf > ./nginx.conf
# build
docker build -t test-nginx .
# start
docker run --name my-test-nginx -p 8080:80 -d test-nginx
# reuse container
docker start my-test-nginx
```

## storybook

[Install Storybook • Storybook docs](https://storybook.js.org/docs/get-started/install)
```bash
npx storybook@latest init
```

Storybookをnginxで配信して、playwrightでVRTする
```bash
# npx story build
npm run build-storybook
# start server
docker start my-test-nginx
# npx playwright test (storybook spec file)
npm run test:e2e story.spec.ts
```
[CLI options • Storybook docs](https://storybook.js.org/docs/api/cli-options#build)


## 今後必要


[Storybookを書くだけでリグレッションテストが 実行される世界へようこそ - Speaker Deck](https://speakerdeck.com/kubotak/storybookwoshu-kudakederiguretusiyontesutoga-shi-xing-sarerushi-jie-heyoukoso?slide=33)
[Continuous Integration | Playwright](https://playwright.dev/docs/ci)


