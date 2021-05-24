# new-blog.ojisan.io

シン・blog.ojisan.io

## dev

```sh
npm install -g sass

sass src/style/post.scss:src/style/post.css src/style/top.scss:src/style/top.css
```

## deploy

### html

deploy

```sh
# TODO: root 使いたくない
rsync -av -e 'ssh -i ~/.ssh/sakura_vps__blog_rsa' public/ root@160.16.68.139:/var/www/html
```

debug

```sh
ssh ojisan@160.16.68.139 -i ~/.ssh/sakura_vps__blog_rsa
```

### nginx conf

deploy

```sh
# TODO: root 使いたくない
rsync -av -e 'ssh -i ~/.ssh/sakura_vps__blog_rsa' server/nginx.conf root@160.16.68.139:/etc/nginx/conf.d/
```
