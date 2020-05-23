from django.urls import path

from . import views

urlpatterns = [
    path("", views.index, name="index"),
    path("pricing", views.pricing, name="pricing"),
    path("faq", views.faq, name="faq"),
    path("mailing-list", views.mailing_list, name="mailing_list"),
]
