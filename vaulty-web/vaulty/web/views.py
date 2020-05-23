from django.http import HttpResponse, HttpResponseRedirect, HttpResponseNotAllowed
from django.shortcuts import render

from .forms import LandingEmailForm


def index(request):
    form = LandingEmailForm()
    return render(request, "web/index.html", {"form": form})


def pricing(request):
    return render(request, "web/pricing.html")


def faq(request):
    return render(request, "web/faq.html")


def mailing_list(request):
    if request.method == "POST":
        form = LandingEmailForm(request.POST)
        if form.is_valid():
            data = form.cleaned_data
            print(data["email"])
            return HttpResponse("Done!")
    else:
        return HttpResponseNotAllowed(["POST"])
