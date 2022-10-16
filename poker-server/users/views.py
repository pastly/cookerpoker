import json
from django.http import HttpResponse
from django.shortcuts import render, redirect
from django.contrib.auth import authenticate, login
from django.contrib.auth.decorators import login_required
from django.contrib.auth.models import User
from .forms import RegisterForm

def register(request):
    if request.method == 'POST':
        form = RegisterForm(request.POST)
        if form.is_valid():
            user = form.save()
            user.refresh_from_db()
            user.save()
            pw = form.cleaned_data.get('password1')
            user = authenticate(username=user.username, password=pw)
            if user is not None:
                login(request, user)
                return redirect('home')
    else:
        form = RegisterForm()
    return render(request, 'users/register.html', {'form': form})

@login_required
def info(request, user_id):
    user = User.objects.filter(id=user_id).values('id', 'username').first()
    return HttpResponse(json.dumps(user), content_type='application/json')
