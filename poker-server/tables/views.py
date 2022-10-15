from django.shortcuts import get_object_or_404, render, redirect
from django.contrib.auth.decorators import login_required
from django.http import HttpResponse, HttpResponseNotAllowed, HttpResponseForbidden
from .forms import NewTableForm
from .models import TableState
from .models import Table

import poker_core_py

def latest_state(table_id):
    states = TableState.objects.filter(table_id=table_id).order_by('-id')[:1]
    if len(states):
        return states[0].data
    state_data = poker_core_py.new_game_state()
    # print('made new state ------- ')
    # print(state_data)
    state = TableState(table=Table.objects.get(pk=table_id), data=state_data)
    state.save()
    return state.data

def latest_filtered_state(table_id, player_id):
    state = latest_state(table_id)
    return poker_core_py.filter_state(state, player_id)

def save_state(table, state_data):
    state = TableState(table=table, data=state_data)
    state.save()

@login_required
def index(request):
    if request.method == 'POST':
        form = NewTableForm(request.POST)
        if form.is_valid():
            table = form.save(commit=False)
            table.owner = request.user
            table.save()
            return redirect('tables:detail', table.id)
    latest_tables = Table.objects.order_by('-creation_date')[:10]
    context = {'latest_tables': latest_tables, 'new_table_form': NewTableForm()}
    return render(request, 'tables/index.html', context)

@login_required
def detail(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    if request.method != 'POST':
        user_is_owner = request.user.id == table.owner.id
        return render(
            request,
            'tables/detail.html',
            {'table': table, 'user_is_owner': user_is_owner})
    if 'join' in request.POST:
        user_id = request.user.id
        table_id = table.id
        stack = 1000
        state = latest_state(table_id)
        new_state = poker_core_py.seat_player(state, user_id, stack)
        save_state(table, new_state)
        return redirect('tables:play', table_id)
    elif 'delete' in request.POST:
        table = get_object_or_404(Table, pk=table_id)
        user = request.user
        if request.method != 'POST':
            return HttpResponseNotAllowed(['POST'])
        if table.owner.id != user.id:
            return HttpResponseForbidden()
        table.delete()
        return redirect('tables:index')


@login_required
def play(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    user = request.user
    # TODO: return page showing table and loading WASM that polls for state updates
    return render(
        request,
        'tables/play.html',
        {
            'state': latest_filtered_state(table.id, user.id),
            'table': table,
        }
    )

    
def state_get(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    # TODO: ensure user is seated at table
    user = request.user
    # try ticking state forward
    state = latest_state(table.id)
    new_state = poker_core_py.tick_state(state)
    save_state(table, new_state)
    # print(latest_filtered_state(table.id, user.id))
    return HttpResponse(latest_filtered_state(table.id, user.id))

def state_post(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    # TODO: ensure user is seated at table
    user = request.user
    state = latest_state(table.id)
    obj = request.body.decode(request.encoding or 'utf-8')
    new_state = poker_core_py.player_action(state, user.id, obj)
    save_state(table, new_state)
    new_state2 = poker_core_py.tick_state(new_state)
    save_state(table, new_state2)
    return HttpResponse(latest_filtered_state(table.id, user.id))

@login_required
def state(request, table_id):
    if request.method == 'POST':
        return state_post(request, table_id)
    return state_get(request, table_id)

@login_required
def method_reset(request, table_id):
    # TODO: ensure user is seated at table
    # TODO: ensure user is admin of table? in future this code should be removed, so maybe not important
    table = get_object_or_404(Table, pk=table_id)
    state = poker_core_py.devonly_reset_state(latest_state(table_id))
    save_state(table, state)
    return redirect('tables:play', table.id)
