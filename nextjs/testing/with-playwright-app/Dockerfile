FROM nginx

COPY nginx/app.conf /etc/nginx/conf.d/app.conf
COPY nginx/nginx.conf /etc/nginx/nginx.conf
COPY storybook-static /usr/share/nginx/html

# maybe this is meaningless.
EXPOSE 80