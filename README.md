# rss-trans

RSSやAtomのタイトルだけを任意の言語に翻訳するツール。

## デプロイ方法

### 前準備

GoogleCloudTranslationを利用しているため、GoogleCloudのアカウントを用意する。

1. CloudTranslationAPIを有効化する。  
    https://cloud.google.com/translate
2. GoogleCloud上でサービスアカウント発行し下記の権限を付与する。
    - Cloud Translation API 閲覧者
    - Service Usage ユーザー
3. 発行したサービスアカウントの鍵(.json)をダウンロードする。


### 実行方法

1. Dockerでビルドする
``` shell
docker build
```

2. 起動する
``` 
docker run -p 7777:8080 \
    -v $(pwd)/service-account.json:/app/service-account.json \
    -e GOOGLE_CLOUD_PROJECT=${YOUR_GCP_PROJECT_ID} ${IMAGE_NAME}
```
- service-account.json
    - 先に発行したサービスアカウントの鍵
- port
    - ローカルの7777にフォワードする
- GOOGLE_CLOUD_PROJECT
    - 先に有効化したCloudTranslationのProjectID
    - name_9999 のようなもの

### 使い方

下記のURLへアクセスすることで、翻訳後のRSS、Atomが取得できる。

`https://example.com/rss?url=${FEED_URL}&to=ja-JP`

### 環境変数

- CACHE_MODE
    - キャッシュモード
    - webdav
        - WebDavによるキャッシュ
    - s3
        - S3によるキャッシュ
- WEB_DAV_URL
    - キャッシュで利用するWebDavのURL
- WEB_DAV_USER_ID
    - キャッシュで利用するWebDavのユーザーID
- WEB_DAV_USER_PASSWORD
    - キャッシュで利用するWebDavのパスワード
- S3_ENDPOINT_URL
    - キャッシュで利用するS3のエンドポイントURL
- S3_BUCKET_NAME
    - キャッシュで利用するS3のバケット名
- S3_REGION
    - キャッシュで利用するS3のリージョン
- AWS_ACCESS_KEY_ID
    - キャッシュで利用するS3のアクセスキー
- AWS_SECRET_ACCESS_KEY
    - キャッシュで利用するS3のシークレットキー
