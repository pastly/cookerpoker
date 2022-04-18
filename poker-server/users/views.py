from django.shortcuts import render, redirect
from django.contrib.auth import authenticate, login
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
