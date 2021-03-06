import os

# Build paths inside the project like this: os.path.join(BASE_DIR, ...)
BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


# Quick-start development settings - unsuitable for production
# See https://docs.djangoproject.com/en/3.0/howto/deployment/checklist/

SECRET_KEY = os.environ.get("VAULTY_WEB_DJANGO_SECRET_KEY",
                            "z4%y2t#e!l-&tb(2s*!u$@$zlw2@=*p)plb4(9#boqsz0@a&r4")

IS_PROD = "VAULTY_WEB_IS_PROD" in os.environ

# If production env flag is present, disable debug mode
DEBUG = not IS_PROD

# We only need localhost here as all requests will be proxied from Nginx
ALLOWED_HOSTS = ["vaulty.net", "localhost"]

# Define a custom User model
# This simply extends the regular Django user with extra fields
AUTH_USER_MODEL = 'web.User'

# Application definition

INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',
    'web',
    'social_django',
]

MIDDLEWARE = [
    'django.middleware.security.SecurityMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
]

ROOT_URLCONF = 'vaulty.urls'

TEMPLATES = [
    {
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
        'DIRS': [],
        'APP_DIRS': True,
        'OPTIONS': {
            'context_processors': [
                'django.template.context_processors.debug',
                'django.template.context_processors.request',
                'django.contrib.auth.context_processors.auth',
                'django.contrib.messages.context_processors.messages',

                # python-social-auth
                'social_django.context_processors.backends',
                'social_django.context_processors.login_redirect',
            ],
        },
    },
]

WSGI_APPLICATION = 'vaulty.wsgi.application'


# Database
# https://docs.djangoproject.com/en/3.0/ref/settings/#databases

DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.postgresql_psycopg2',
        'NAME': os.environ.get("VAULTY_WEB_DB_NAME", None),
        'USER': os.environ.get("VAULTY_WEB_DB_USER", None),
        'PASSWORD': os.environ.get("VAULTY_WEB_DB_PASS", ""),
        'HOST': os.environ.get("VAULTY_WEB_DB_HOST", None),
        'PORT': '',
    }
}

if IS_PROD:
    CSRF_COOKIE_SECURE = True
    SESSION_COOKIE_SECURE = True

# Email
EMAIL_HOST = os.environ.get("VAULTY_WEB_MAIL_HOST")
EMAIL_HOST_USER = os.environ.get("VAULTY_WEB_MAIL_USER")
EMAIL_HOST_PASSWORD = os.environ.get("VAULTY_WEB_MAIL_PASSWORD")
EMAIL_PORT = 587
EMAIL_USE_TLS = True

# Password validation
# https://docs.djangoproject.com/en/3.0/ref/settings/#auth-password-validators

AUTH_PASSWORD_VALIDATORS = [
    {
        'NAME': 'django.contrib.auth.password_validation.UserAttributeSimilarityValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.MinimumLengthValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.CommonPasswordValidator',
    },
    {
        'NAME': 'django.contrib.auth.password_validation.NumericPasswordValidator',
    },
]

AUTHENTICATION_BACKENDS = (
    'django.contrib.auth.backends.ModelBackend',

    # python-social-auth backends
    'social_core.backends.dropbox.DropboxOAuth2V2',
)


# Internationalization
# https://docs.djangoproject.com/en/3.0/topics/i18n/

LANGUAGE_CODE = 'en-us'

TIME_ZONE = 'UTC'

USE_I18N = True

USE_L10N = True

USE_TZ = True


# Static files (CSS, JavaScript, Images)
# https://docs.djangoproject.com/en/3.0/howto/static-files/

STATIC_URL = '/static/'
STATIC_ROOT = os.path.join(BASE_DIR, 'static/')

# python-social-auth
SOCIAL_AUTH_POSTGRES_JSONFIELD = True
SOCIAL_AUTH_USER_MODEL = 'web.User'
