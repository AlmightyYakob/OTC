import ComponentOne from '@/components/ComponentOne.vue';
import ComponentTwo from '@/components/ComponentTwo.vue';
const test = [
    'a'
];
export default defineComponent({
    components: {
        ComponentOne,
        ComponentTwo
    },
    props: {
        id: {
            type: String,
            required: true
        }
    },
    setup (props, ctx) {
        const something = inject('something');
        const something2 = inject('something2', ()=>3);
        const otherInject = inject("otherInject", 'some default');
        const noDefaultInject = inject('something3');
        const loading = ref(false);
        const foo = ref(null);
        const count = ref(0);
        const headers = ref([
            {
                text: 'Name',
                value: 'name'
            },
            {
                text: 'Identifier',
                value: 'identifier'
            }, 
        ]);
        const bar = computed(()=>foo.value || props.id);
        const baz = computed(()=>{
            if (loading.value) {
                return 0;
            }
            return count.value + 1;
        });
        watch(loading, (val, oldVal)=>{
            if (val === true) {
                console.log('now loading!');
                foo.value = 2;
            }
        });
        watch(headers, (val)=>{
            console.log('headers changed', val);
        }, {
            deep: true,
            immediate: true
        });
        foo.value = 1;
        ctx.$emit('emission');
        function method1(arg) {
            console.log(arg);
        }
        async function method2() {
            console.log('async!');
            something.foo.bar.run();
        }
        onMounted(async ()=>{
            loading.value = true;
            method1(count.value);
            count.value += 1;
            loading.value = false;
        });
        return {
            something,
            something2,
            otherInject,
            noDefaultInject,
            loading,
            foo,
            count,
            headers,
            bar,
            baz,
            method1,
            method2
        };
    }
});
