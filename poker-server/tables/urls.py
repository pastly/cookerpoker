from django.urls import path

from . import views

app_name = 'tables'
urlpatterns = [
    path('', views.index, name='index'),
    path('<int:table_id>/', views.detail, name='detail'),
]
