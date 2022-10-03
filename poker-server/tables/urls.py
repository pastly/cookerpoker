from django.urls import path

from . import views

app_name = 'tables'
urlpatterns = [
    path('', views.index, name='index'),
    path('<int:table_id>/', views.detail, name='detail'),
    path('<int:table_id>/play', views.play, name='play'),
    path('<int:table_id>/state', views.state, name='state'),
    path('<int:table_id>/play/reset', views.method_reset, name='method_reset'),
]
