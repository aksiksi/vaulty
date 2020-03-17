from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name="index"),
    path("mailing-list", views.mailing_list, name="mailing_list"),
]
