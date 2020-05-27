from django.contrib import admin
from django.urls import include, path

urlpatterns = [
    path('admin/', admin.site.urls),
    path("", include("web.urls")),
    path("social/", include("social_django.urls"))
]
